use std::collections::HashMap;

use z3::ast::{Bool, Real};

use crate::{
    problem::{self, ObjectSet, Problem, Solution},
    SolverError,
};

// A state is a choice between several possible tokens
// in the sequence of values that make up a timeline.
struct State<'z> {
    start_time: Real<'z>,
    end_time: Real<'z>,
    timeline: usize,
    tokens: Vec<usize>,
}

struct Token<'a, 'z> {
    active: Option<Bool<'z>>,
    state: usize,
    value: &'a str,
}

struct Condition<'a, 'z3> {
    token_idx: usize,
    cond_spec: &'a problem::Condition,
    token_queue: usize,
    alternatives_extension: Option<Bool<'z3>>,
}

struct Timeline {
    states: Vec<usize>,
}

pub fn solve(problem: &Problem) -> Result<Solution, SolverError> {
    let z3_config = z3::Config::new();
    let ctx = z3::Context::new(&z3_config);
    let solver = z3::Solver::new(&ctx);

    let groups_by_name = problem
        .groups
        .iter()
        .map(|g| (g.name.as_str(), &g.members))
        .collect::<HashMap<_, _>>();

    let mut timelines = problem
        .timelines
        .iter()
        .map(|t| Timeline { states: Vec::new() })
        .collect::<Vec<_>>();

    let mut states = Vec::new();
    let mut states_queue = 0;
    let mut tokens = Vec::new();
    let mut tokens_queue = 0;
    let mut conds = Vec::new();
    let mut conds_queue = 0;

    let mut timelines_by_name = problem
        .timelines
        .iter()
        .enumerate()
        .map(|(i, t)| (t.name.as_str(), i))
        .collect::<HashMap<_, _>>();

    // STATIC TOKENS
    // The facts need to be the first states.
    for const_token in problem.tokens.iter() {
        if !timelines_by_name.contains_key(const_token.timeline_name.as_str()) {
            timelines_by_name.insert(const_token.timeline_name.as_str(), timelines.len());
            timelines.push(Timeline { states: Vec::new() });
        }
    }

    for const_token in problem.tokens.iter() {
        if let crate::problem::TokenTime::Fact(start_time, end_time) = const_token.const_time {
            if !timelines[timelines_by_name[const_token.timeline_name.as_str()]]
                .states
                .is_empty()
            {
                todo!("Multiple facts.");
            }

            let token_idx = tokens.len();
            let state_idx = states.len();
            tokens.push(Token {
                active: None,
                value: &const_token.value,
                state: state_idx,
            });
            states.push(State {
                tokens: vec![token_idx],
                start_time: start_time
                    .map(|t| Real::from_real(&ctx, t as i32, 1))
                    .unwrap_or_else(|| Real::fresh_const(&ctx, "t")),
                end_time: end_time
                    .map(|t| Real::from_real(&ctx, t as i32, 1))
                    .unwrap_or_else(|| Real::fresh_const(&ctx, "t")),
                timeline: timelines_by_name[const_token.timeline_name.as_str()],
            });
            timelines[timelines_by_name[const_token.timeline_name.as_str()]]
                .states
                .push(state_idx);
        }
    }

    // All empty timelines must now start in one of their initial states.
    for timeline in 0..timelines.len() {
        if timelines[timeline].states.is_empty() {
            assert!(timeline < problem.timelines.len());

            let state_idx = states.len();
            let values = next_values_from(&problem.timelines[timeline], None);

            let state_tokens = values
                .into_iter()
                .map(|value| Token {
                    active: Some(Bool::fresh_const(&ctx, "x")),
                    state: state_idx,
                    value,
                })
                .collect::<Vec<_>>();

            // At most one state can be chosen.
            let am1 = state_tokens
                .iter()
                .filter_map(|t| t.active.as_ref().map(|b| (b, 1)))
                .collect::<Vec<_>>();
            solver.assert(&Bool::pb_le(&ctx, &am1, 1));

            tokens.extend(state_tokens);
        }
    }

    for const_token in problem.tokens.iter() {
        if let crate::problem::TokenTime::Goal = const_token.const_time {
            let token_idx = tokens.len();
            let state_idx = states.len();
            tokens.push(Token {
                active: None, // const true
                value: &const_token.value,
                state: state_idx,
            });
            states.push(State {
                start_time: Real::fresh_const(&ctx, "t"),
                end_time: Real::fresh_const(&ctx, "t"),
                tokens: vec![token_idx],
                timeline: timelines_by_name[const_token.timeline_name.as_str()],
            });
            timelines[timelines_by_name[const_token.timeline_name.as_str()]]
                .states
                .push(state_idx);
        }
    }

    // REFINEMENT LOOP
    '_refinement: loop {
        // EXPAND PROBLEM FORMULATION

        while states_queue < states.len() || tokens_queue < tokens.len() || conds_queue < conds.len() {
            while states_queue < states.len() {
                let state_idx = states_queue;
                states_queue += 1;
            }

            while tokens_queue < tokens.len() {
                let token_idx = tokens_queue;
                tokens_queue += 1;

                let value_spec = problem.timelines[states[tokens[token_idx].state].timeline]
                    .values
                    .iter()
                    .find(|s| s.name == tokens[token_idx].value)
                    .unwrap();

                // Minimum duration of state.
                let prec = &Real::le(
                    &Real::add(
                        &ctx,
                        &[
                            &states[tokens[token_idx].state].start_time,
                            &Real::from_real(&ctx, value_spec.duration.0 as i32, 1),
                        ],
                    ),
                    &states[tokens[token_idx].state].end_time,
                );
                if let Some(cond) = tokens[token_idx].active.as_ref() {
                    solver.assert(&Bool::implies(cond, prec))
                } else {
                    solver.assert(prec);
                }

                for cond_spec in value_spec.conditions.iter() {
                    conds.push(Condition {
                        token_idx,
                        token_queue: 0,
                        cond_spec,
                        alternatives_extension: None,
                    });
                }
            }

            while conds_queue < conds.len() {
                let cond_idx = conds_queue;
                conds_queue += 1;
                // The Condition is unprocessed. This means that it is either new; or it has been
                // placed on the list of updated conditions for a reason.
                let need_new_token = true;

                let objects: Vec<&str> = match &conds[cond_idx].cond_spec.object {
                    ObjectSet::Group(c) => groups_by_name
                        .get(c.as_str())
                        .iter()
                        .flat_map(|c| c.iter().map(String::as_str))
                        .collect::<Vec<_>>(),
                    ObjectSet::Object(n) => {
                        vec![n.as_str()]
                    }
                };

                let mut alternatives = Vec::new();

                let mut all_target_tokens = Vec::new();
                let mut new_target_tokens = Vec::new();
                for obj in objects.iter() {
                    let timeline_idx = timelines_by_name[obj];
                    let matching_tokens = tokens.iter().enumerate().filter(|(_, t)| {
                        states[t.state].timeline == timeline_idx && t.value == conds[cond_idx].cond_spec.value
                    });
                    for (token, _) in matching_tokens {
                        all_target_tokens.push(token);

                        if token >= conds[cond_idx].token_queue {
                            new_target_tokens.push(token);
                        }
                    }
                }

                if need_new_token && new_target_tokens.is_empty() {
                    // Select a object at random
                    // TODO make a heuristic for this

                    #[allow(clippy::never_loop)] // TODO multiplicity check will allow this to loop
                    for i in 0..objects.len() {
                        // if let Some(obj_name) = objects {

                        // This is a "random" heuristic for which object to expand.
                        let selected_object = (tokens.len() + conds.len() + i) % objects.len();
                        let obj_name = objects[selected_object];

                        // Can we generate a new token in this timeline?

                        println!(
                            "Finding new states to add to get to {}.{}",
                            obj_name, conds[cond_idx].cond_spec.value
                        );
                        let tl_idx = timelines_by_name[obj_name];
                        if let Some(distance) = distance_to(
                            &problem.timelines[tl_idx],
                            &timelines[tl_idx],
                            conds[cond_idx].cond_spec.value,
                        ) {
                            // Expand the state space.
                            for _ in 0..distance {
                                let values = next_values_from(todo!(), todo!());
                            }
                        } else {
                            println!("Cannot get to that state from here.");
                            panic!();
                        }
                    }
                }
            }
        }
    }
}

fn next_values_from<'a>(timeline: &'a problem::Timeline, values: Option<&[&'a str]>) -> Vec<&'a str> {
    timeline
        .values
        .iter()
        .filter_map(|v| {
            v.conditions
                .iter()
                .any(|c| {
                    c.object == ObjectSet::Object(timeline.name.clone())
                        && matches!(c.temporal_relationship, problem::TemporalRelationship::Meet)
                })
                .then(|| v.name.as_str())
        })
        .collect::<Vec<_>>()
}

fn distance_to(tl_spec: &problem::Timeline, tl: &Timeline, value: String) -> Option<usize> {
    todo!()
}
