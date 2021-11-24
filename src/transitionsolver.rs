use std::collections::{HashMap, HashSet};

use z3::ast::{Ast, Bool, Real};

use crate::{
    from_z3_real,
    problem::{self, ObjectSet, Problem, Solution, SolutionToken, TemporalRelationship, TokenTime},
    SolverError,
};

// A state is a choice between several possible tokens
// in the sequence of values that make up a timeline.
struct State<'z> {
    start_time: Real<'z>,
    end_time: Real<'z>,
    timeline: usize,
    tokens: Vec<usize>,
    state_seq: usize,
    active: Bool<'z>,
    activate_next: Bool<'z>,
}

struct Token<'a, 'z> {
    active: Option<Bool<'z>>,
    state: usize,
    value: &'a str,
    fact: bool,
}

struct Condition<'a, 'z3> {
    token_idx: usize,
    cond_spec: &'a problem::Condition,
    token_queue: usize,
    alternatives_extension: Option<Bool<'z3>>,
}

struct Timeline<'z> {
    states: Vec<usize>,
    goal_state_extension: Option<Bool<'z>>,
    facts_only: bool,
}

pub fn solve(problem: &Problem) -> Result<Solution, SolverError> {
    let _p = hprof::enter("solve");
    let p1 = hprof::enter("prepare");
    // println!("Starting transition-and-pocl solver.");
    let z3_config = z3::Config::new();
    let ctx = z3::Context::new(&z3_config);
    let solver = z3::Solver::new(&ctx);

    let mut params = z3::Params::new(&ctx);
    params.set_bool("auto_config", false);
    params.set_u32("smt.arith.solver", 6);
    // println!("Z3 params:\n{}", params.to_string());
    solver.set_params(&params);

    let groups_by_name = problem
        .groups
        .iter()
        .map(|g| (g.name.as_str(), &g.members))
        .collect::<HashMap<_, _>>();

    let mut timelines = problem
        .timelines
        .iter()
        .map(|_| Timeline {
            states: Vec::new(),
            goal_state_extension: None,
            facts_only: false,
        })
        .collect::<Vec<_>>();

    let mut timeline_names = problem.timelines.iter().map(|t| t.name.as_str()).collect::<Vec<_>>();

    let mut states = Vec::new();
    let mut states_queue = 0;
    let mut tokens = Vec::new();
    let mut tokens_queue = 0;
    let mut conds: Vec<Condition> = Vec::new();
    let mut conds_queue = 0;

    let mut goal_lits: HashMap<(&str, isize), Bool> = HashMap::new();

    let mut expand_links_queue: Vec<(bool, usize)> = Vec::new();

    let mut expand_links_lits: HashMap<Bool, usize> = HashMap::new();
    let mut expand_goal_state_lits: HashMap<Bool, usize> = HashMap::new();

    let mut resource_constraints: HashMap<usize, ResourceConstraint> = Default::default(); // token to resourceconstraint

    let mut timelines_by_name = problem
        .timelines
        .iter()
        .enumerate()
        .map(|(i, t)| (t.name.as_str(), i))
        .collect::<HashMap<_, _>>();

    // STATIC TOKENS

    // Add timelines for timelines that don't have a timeline specification, but still has facts (simple fact timelines).
    for const_token in problem.tokens.iter() {
        if !timelines_by_name.contains_key(const_token.timeline_name.as_str()) {
            timelines_by_name.insert(const_token.timeline_name.as_str(), timelines.len());
            timeline_names.push(const_token.timeline_name.as_str());
            timelines.push(Timeline {
                states: Vec::new(),
                goal_state_extension: None,
                facts_only: true,
            });

            assert!(timeline_names.len() == timelines.len());
            assert!(timelines_by_name.len() == timelines.len());
        }
    }

    // The facts need to be the first states.
    for const_token in problem.tokens.iter() {
        if let crate::problem::TokenTime::Fact(start_time, end_time) = const_token.const_time {
            if !timelines[timelines_by_name[const_token.timeline_name.as_str()]]
                .states
                .is_empty()
            {
                // todo!("Multiple facts.");
            }

            let token_idx = tokens.len();
            let state_idx = states.len();
            tokens.push(Token {
                active: None,
                value: &const_token.value,
                state: state_idx,
                fact: true,
            });
            states.push(State {
                state_seq: 0,
                tokens: vec![token_idx],
                start_time: start_time
                    .map(|t| Real::from_real(&ctx, t as i32, 1))
                    .unwrap_or_else(|| Real::fresh_const(&ctx, "t")),
                end_time: end_time
                    .map(|t| Real::from_real(&ctx, t as i32, 1))
                    .unwrap_or_else(|| Real::fresh_const(&ctx, "t")),
                timeline: timelines_by_name[const_token.timeline_name.as_str()],
                active: Bool::from_bool(&ctx, true),
                activate_next: Bool::fresh_const(&ctx, "nxstate"),
            });
            timelines[timelines_by_name[const_token.timeline_name.as_str()]]
                .states
                .push(state_idx);

            // Facts can have capacities
            resource_constraints.entry(token_idx).or_default().capacity = Some(const_token.capacity);
        }
    }

    // All empty timelines must now start in one of their initial states.
    for timeline in 0..timelines.len() {
        if timelines[timeline].states.is_empty() {
            assert!(timeline < problem.timelines.len());

            let expanded = expand_until(
                problem,
                &ctx,
                &solver,
                timeline,
                &mut timelines,
                &mut states,
                &mut tokens,
                None,
            );
            assert!(expanded);
        }
    }

    // TODO :: this gives a perf boost on GOAC isntances
    // because we don't need to find so many UNSAT.
    // Could do a pidgeonhole argument for all the constant links to the same timeline,
    //   and expand this from the beginning?

    // for timeline in 0..timelines.len() {
    //     if  timeline_names[timeline] == "loc" {
    //         let expanded = expand_n(
    //             problem,
    //             &ctx,
    //             &solver,
    //             timeline,
    //             &mut timelines,
    //             &mut states,
    //             &mut tokens,
    //             16,
    //         );
    //     }
    // }

    let mut n_smt_calls = 0;

    // println!("TL names {:?}", timelines_by_name);

    drop(p1);

    // REFINEMENT LOOP
    '_refinement: loop {
        // EXPAND PROBLEM FORMULATION

        while states_queue < states.len()
            || tokens_queue < tokens.len()
            || conds_queue < conds.len()
            || !expand_links_queue.is_empty()
        {
            let p = hprof::enter("expand_states");

            while states_queue < states.len() {
                let state_idx = states_queue;
                states_queue += 1;

                // Does this timeline have a goal state?
                let facts_only = timelines[states[state_idx].timeline].facts_only;
                // println!(
                //     "Expanding state {} timeline {} (factsonly={})",
                //     state_idx, states[state_idx].timeline, facts_only
                // );

                // There are no goals for facts only timelines.
                if !facts_only {
                    let timeline_idx = states[state_idx].timeline;
                    let timeline_name = timeline_names[timeline_idx];
                    if let Some(goal) = problem.tokens.iter().find(|const_token| {
                        const_token.timeline_name == timeline_name && matches!(const_token.const_time, TokenTime::Goal)
                    }) {
                        // Is this a potential final/goal state?
                        if let Some(token_idx) = states[state_idx]
                            .tokens
                            .iter()
                            .find(|t| tokens[**t].value == goal.value)
                        {
                            let can_expand = {
                                let timeline = &timelines[timeline_idx];
                                !timeline.facts_only
                                    && can_expand(
                                        &problem.timelines[timeline_idx],
                                        &states[state_idx]
                                            .tokens
                                            .iter()
                                            .map(|t| tokens[*t].value)
                                            .collect::<Vec<_>>(),
                                        &goal.value,
                                    )
                            };

                            if can_expand {
                                let goal_lit = Bool::fresh_const(&ctx, "goal");
                                if let Some(active) = tokens[*token_idx].active.as_ref() {
                                    solver.assert(&Bool::implies(&goal_lit, active));
                                }
                                assert!(goal_lits
                                    .insert((timeline_name, states[state_idx].state_seq as isize), goal_lit.clone())
                                    .is_none());

                                // Select at least one goal (at most one goal is implied by the disabling of tokens below)
                                let mut clause = Vec::new();
                                if let Some(prev_extension) = timelines[timelines_by_name[timeline_name]]
                                    .goal_state_extension
                                    .as_ref()
                                {
                                    assert!(expand_goal_state_lits.remove(prev_extension).is_some());
                                    clause.push(Bool::not(prev_extension));
                                }
                                clause.push(goal_lit);

                                let extension = Bool::fresh_const(&ctx, "addgoal");
                                clause.push(extension.clone());
                                expand_goal_state_lits.insert(extension.clone(), timelines_by_name[timeline_name]);
                                timelines[timelines_by_name[timeline_name]].goal_state_extension = Some(extension);

                                let clause_refs = clause.iter().collect::<Vec<_>>();
                                solver.assert(&Bool::or(&ctx, &clause_refs));
                            } else {
                                // If we can only get to the goal one time, then
                                // might as well just assert this state.
                                if let Some(active) = tokens[*token_idx].active.as_ref() {
                                    solver.assert(active);
                                }
                            }
                        }
                    }

                    // Does the previous state have a goal lit?
                    if let Some(goal_in_prev_state) =
                        goal_lits.get(&(timeline_name, states[state_idx].state_seq as isize - 1))
                    {
                        // Disable each possible token, if the previous state was a goal state.
                        solver.assert(&Bool::implies(
                            goal_in_prev_state,
                            &Bool::not(&states[state_idx].active),
                        ));
                    }

                    // Did we imply that the next state has to be active (from the previous one)
                    if states[state_idx].state_seq > 0 {
                        let prev_state_idx =
                            timelines[states[state_idx].timeline].states[states[state_idx].state_seq - 1];
                        solver.assert(&Bool::implies(
                            &states[prev_state_idx].activate_next,
                            &states[state_idx].active,
                        ));
                    }

                    // Does the previous state have forward transition conditions?
                    if states[state_idx].state_seq > 0 {
                        let prev_state_idx =
                            timelines[states[state_idx].timeline].states[states[state_idx].state_seq - 1];

                        for source_token_idx in states[prev_state_idx].tokens.iter().copied() {
                            if let Some(value_spec) = problem.timelines[states[tokens[source_token_idx].state].timeline]
                                .values
                                .iter()
                                .find(|s| s.name == tokens[source_token_idx].value)
                            {
                                for cond in value_spec.conditions.iter() {
                                    if let Some(next_value) = cond.is_timeline_transition_to(
                                        &problem.timelines[states[tokens[source_token_idx].state].timeline].name,
                                    ) {
                                        // If token from preivous state is active...

                                        let mut clause = Vec::new();
                                        if let Some(active) = tokens[source_token_idx].active.as_ref() {
                                            clause.push(Bool::not(active));
                                        }

                                        // ... then the current state must have the given value.
                                        // println!(
                                        //     "find next value {:?} {}.{}->{}", cond,
                                        //     problem.timelines[states[tokens[source_token_idx].state].timeline].name,
                                        //     value_spec.name,
                                        //     next_value
                                        // );
                                        let goal_token_idx = states[state_idx]
                                            .tokens
                                            .iter()
                                            .find(|t| tokens[**t].value == next_value)
                                            .unwrap();

                                        if let Some(active) = tokens[*goal_token_idx].active.as_ref() {
                                            clause.push(active.clone());
                                        }

                                        if !clause.is_empty() {
                                            let clause_refs = clause.iter().collect::<Vec<_>>();
                                            solver.assert(&Bool::or(&ctx, &clause_refs));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            drop(p);
            let p = hprof::enter("expand_tokens");
            while tokens_queue < tokens.len() {
                let token_idx = tokens_queue;
                tokens_queue += 1;

                if tokens[token_idx].fact {
                    // Minimum duration of state.
                    let prec = &Real::le(
                        &Real::add(
                            &ctx,
                            &[
                                &states[tokens[token_idx].state].start_time,
                                &Real::from_real(&ctx, 1_i32, 1),
                            ],
                        ),
                        &states[tokens[token_idx].state].end_time,
                    );
                    solver.assert(prec);
                } else {
                    // println!("EXPANDING TOKEN {}.{}", timeline_names[states[tokens[token_idx].state].timeline], tokens[token_idx].value);
                    let value_spec = problem.timelines[states[tokens[token_idx].state].timeline]
                        .values
                        .iter()
                        .find(|s| s.name == tokens[token_idx].value)
                        .unwrap();

                    resource_constraints.entry(token_idx).or_default().capacity = Some(value_spec.capacity);

                    // If there are old links pointing to this value, we need to update them.
                    // println!("Adding links for {}.{}", token.timeline_name, token.value);
                    for (cond_idx, cond) in conds.iter().enumerate() {
                        if cond.cond_spec.value == tokens[token_idx].value {
                            expand_links_queue.push((false, cond_idx));
                        }
                    }

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

                    // Maximum duration
                    if let Some(max_dur) = value_spec.duration.1 {
                        let prec = &Real::ge(
                            &Real::add(
                                &ctx,
                                &[
                                    &states[tokens[token_idx].state].start_time,
                                    &Real::from_real(&ctx, max_dur as i32, 1),
                                ],
                            ),
                            &states[tokens[token_idx].state].end_time,
                        );

                        if let Some(cond) = tokens[token_idx].active.as_ref() {
                            solver.assert(&Bool::implies(cond, prec))
                        } else {
                            solver.assert(prec);
                        }
                    }

                    for cond_spec in value_spec.conditions.iter() {
                        // is this a timeline transition?
                        if cond_spec
                            .is_timeline_transition_from(
                                &problem.timelines[states[tokens[token_idx].state].timeline].name,
                            )
                            .is_some()
                        {
                            if states[tokens[token_idx].state].state_seq > 0 {
                                let prev_state_seq = states[tokens[token_idx].state].state_seq - 1;
                                let timeline = &timelines[states[tokens[token_idx].state].timeline];
                                let prev_state = &states[timeline.states[prev_state_seq]];

                                // find matching states
                                let matching_states = prev_state
                                    .tokens
                                    .iter()
                                    .filter_map(|t| (tokens[*t].value == cond_spec.value).then(|| &tokens[*t].active));

                                let mut clause = vec![];
                                if let Some(l) = tokens[token_idx].active.as_ref() {
                                    clause.push(Bool::not(l));
                                }

                                let mut any_const = false;
                                let mut n_lits = 0;
                                for m in matching_states {
                                    if let Some(l) = m {
                                        clause.push(l.clone());
                                        n_lits += 1;
                                    } else {
                                        any_const = true;
                                    }
                                }

                                assert!(any_const == (n_lits == 0));

                                if !any_const {
                                    let clause_refs = clause.iter().collect::<Vec<_>>();
                                    solver.assert(&Bool::or(&ctx, &clause_refs));
                                }
                            } else {
                                // println!(
                                //     "No transition condition for initial state for {}",
                                //     &problem.timelines[states[tokens[token_idx].state].timeline].name
                                // );
                            }
                        } else if cond_spec
                            .is_timeline_transition_to(
                                &problem.timelines[states[tokens[token_idx].state].timeline].name,
                            )
                            .is_some()
                        {
                            // Pass, this is handled when adding the next state.
                        } else {
                            // When it's not a timeline transition, make a causal link.
                            conds.push(Condition {
                                token_idx,
                                token_queue: 0,
                                cond_spec,
                                alternatives_extension: None,
                            });
                        }
                    }
                }
            }

            drop(p);
            let p = hprof::enter("expand_conds");
            while conds_queue < conds.len() || !expand_links_queue.is_empty() {
                let (need_new_token, cond_idx) = if conds_queue < conds.len() {
                    let cond_idx = conds_queue;
                    conds_queue += 1;
                    (true, cond_idx)
                } else {
                    expand_links_queue.pop().unwrap()
                };

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

                // let mut all_target_tokens = Vec::new();
                // println!("Finding tokens for object set {:?}", &conds[cond_idx].cond_spec.object);
                let mut new_target_tokens = Vec::new();
                for obj in objects.iter() {
                    // println!("Finding tokens for {}.{}", obj, conds[cond_idx].cond_spec.value);
                    let timeline_idx = timelines_by_name[obj];
                    let matching_tokens = tokens.iter().enumerate().filter(|(_, t)| {
                        states[t.state].timeline == timeline_idx && t.value == conds[cond_idx].cond_spec.value
                    });
                    for (token, _) in matching_tokens {
                        // all_target_tokens.push(token);

                        if token >= conds[cond_idx].token_queue {
                            // println!("  new token {:?}", tokens[token].value);
                            new_target_tokens.push(token);
                        }
                    }
                }

                if need_new_token && new_target_tokens.is_empty() {
                    for i in 0..objects.len() {
                        // This is a "random" (though deterministic) heuristic for which object to expand.
                        let selected_object = (tokens.len() + conds.len() + i) % objects.len();
                        let obj_name = objects[selected_object];

                        // println!(
                        //     "Finding new states to add to get to {}.{}",
                        //     obj_name, conds[cond_idx].cond_spec.value
                        // );

                        let prev_tokens_len = tokens.len();
                        if expand_until(
                            problem,
                            &ctx,
                            &solver,
                            timelines_by_name[obj_name],
                            &mut timelines,
                            &mut states,
                            &mut tokens,
                            Some(&conds[cond_idx].cond_spec.value),
                        ) {
                            assert!(
                                tokens[prev_tokens_len..]
                                    .iter()
                                    .filter(|t| t.value == conds[cond_idx].cond_spec.value.as_str())
                                    .count()
                                    == 1
                            );

                            new_target_tokens.push(
                                prev_tokens_len
                                    + tokens[prev_tokens_len..]
                                        .iter()
                                        .position(|t| (t.value == conds[cond_idx].cond_spec.value.as_str()))
                                        .unwrap(),
                            );

                            // println!("Added token {:?}", new_target_tokens.last());
                            // let token = &tokens[*new_target_tokens.last().unwrap()];
                            // println!("  token state {:?} value {:?}", token.state, token.value);

                            break;
                        } else {
                            // println!("Could not expand.");
                        }
                    }
                }

                if new_target_tokens.is_empty() {
                    if need_new_token && conds[cond_idx].alternatives_extension.is_none() {
                        // Couldn't generate the first token, this condition can never be fulfilled.
                        // println!(
                        //     "unsatisfiable condition {:?} in token {}@{}",
                        //     conds[cond_idx].cond_spec,
                        //     tokens[conds[cond_idx].token_idx].value,
                        //     timeline_names[states[tokens[conds[cond_idx].token_idx].state].timeline],
                        // );

                        if let Some(active) = tokens[conds[cond_idx].token_idx].active.as_ref() {
                            solver.assert(&active.not());
                        } else {
                            println!("The token was constant, problem is unsat.");
                            return Err(SolverError::NoSolution);
                        }
                    }
                } else {
                    let mut alternatives = Vec::new();

                    let old_expansion_lit: Option<Bool> = conds[cond_idx].alternatives_extension.take();

                    if let Some(b) = old_expansion_lit.as_ref() {
                        assert!(expand_links_lits.remove(b).is_some());
                    }

                    let can_expand = objects.iter().any(|t| {
                        let idx = timelines_by_name[t];
                        let timeline = &timelines[idx];

                        !timeline.facts_only
                            && can_expand(
                                &problem.timelines[idx],
                                &states[*timeline.states.last().unwrap()]
                                    .tokens
                                    .iter()
                                    .map(|t| tokens[*t].value)
                                    .collect::<Vec<_>>(),
                                &conds[cond_idx].cond_spec.value,
                            )
                    });

                    // println!(
                    //     "{:?}.{} can_expand={}",
                    //     objects, &conds[cond_idx].cond_spec.value, can_expand
                    // );

                    if can_expand {
                        let expand_lit = Bool::fresh_const(&ctx, "exp");
                        assert!(expand_links_lits.insert(expand_lit.clone(), cond_idx).is_none());
                        conds[cond_idx].alternatives_extension = Some(expand_lit.clone());
                        // println!("added expand lit");
                        alternatives.push(expand_lit);
                    }

                    let need_alternatives = old_expansion_lit
                        .clone()
                        .or_else(|| tokens[conds[cond_idx].token_idx].active.clone());

                    if let Some(cond) = need_alternatives {
                        // println!("added need alternatives {:?}", old_expansion_lit);
                        alternatives.push(Bool::not(&cond));
                    }

                    let const_link = alternatives.len() + new_target_tokens.len() == 1;
                    for token_idx in new_target_tokens.iter().copied() {
                        // Represents the usage of the causal link.
                        let choose_link = (!const_link).then(|| Bool::fresh_const(&ctx, "cl"));

                        let temporal_rel = match conds[cond_idx].cond_spec.temporal_relationship {
                            TemporalRelationship::MetByTransitionFrom => {
                                // // The target token should have a next value to transition to.
                                vec![
                                    states[tokens[token_idx].state].activate_next.clone(),
                                    Real::_eq(
                                        &states[tokens[token_idx].state].end_time,
                                        &states[tokens[conds[cond_idx].token_idx].state].start_time,
                                    ),
                                ]
                            }
                            TemporalRelationship::MetBy => vec![Real::_eq(
                                &states[tokens[token_idx].state].end_time,
                                &states[tokens[conds[cond_idx].token_idx].state].start_time,
                            )],
                            TemporalRelationship::Cover => vec![
                                Real::le(
                                    &states[tokens[token_idx].state].start_time,
                                    &states[tokens[conds[cond_idx].token_idx].state].start_time,
                                ),
                                Real::le(
                                    &states[tokens[conds[cond_idx].token_idx].state].end_time,
                                    &states[tokens[token_idx].state].end_time,
                                ),
                            ],
                            TemporalRelationship::Meets => vec![Real::_eq(
                                &states[tokens[token_idx].state].start_time,
                                &states[tokens[conds[cond_idx].token_idx].state].end_time,
                            )],
                        };

                        if conds[cond_idx].cond_spec.amount > 0 {
                            let rc = resource_constraints.entry(token_idx).or_default();
                            assert!(!rc.closed);
                            rc.users.push((
                                choose_link.clone(),
                                conds[cond_idx].token_idx,
                                conds[cond_idx].cond_spec.amount,
                            ));
                        }

                        // The choose_link boolean implies all the condntions.
                        let mut clause = temporal_rel;
                        if let Some(active) = tokens[token_idx].active.as_ref() {
                            clause.push(active.clone());
                        }

                        for cond in clause {
                            if let Some(choose_link) = choose_link.as_ref() {
                                solver.assert(&Bool::implies(choose_link, &cond));
                                // alternatives.push(choose_link.clone());
                            } else {
                                solver.assert(&cond);
                            }
                        }

                        if let Some(choose_link) = choose_link.as_ref() {
                            alternatives.push(choose_link.clone());
                        }
                    }

                    // println!(
                    //     "TOKEN LINKS for {}.{}[{}] has {} alternatives ({} target tokens)",
                    //     timeline_names[states[tokens[conds[cond_idx].token_idx].state].timeline],
                    //     tokens[conds[cond_idx].token_idx].value,
                    //     conds[cond_idx].token_idx,
                    //     alternatives.len(),
                    //     new_target_tokens.len(),
                    // );

                    assert!(alternatives.is_empty() == const_link);

                    if !alternatives.is_empty() {
                        let alternatives_refs = alternatives.iter().collect::<Vec<_>>();
                        solver.assert(&Bool::or(&ctx, &alternatives_refs));
                    }
                }
                conds[cond_idx].token_queue = tokens.len();
            }

            // every time we touch something, make sure that the timeline transitions are extended all the way to a goal state.

            for const_token in problem.tokens.iter() {
                if let crate::problem::TokenTime::Goal = const_token.const_time {
                    let timeline_idx = timelines_by_name[const_token.timeline_name.as_str()];
                    let timeline = &timelines[timeline_idx];
                    let last_state = timeline.states.last().unwrap();
                    let has_goal = states[*last_state]
                        .tokens
                        .iter()
                        .any(|t| tokens[*t].value == const_token.value);
                    if !has_goal {
                        // println!(
                        //     "Timeline {} has no final goal state. Adding.",
                        //     const_token.timeline_name
                        // );
                        let expanded = expand_until(
                            problem,
                            &ctx,
                            &solver,
                            timeline_idx,
                            &mut timelines,
                            &mut states,
                            &mut tokens,
                            Some(const_token.value.as_str()),
                        );

                        assert!(expanded, "could not expand timeline until goal.");
                    }
                }
            }
        }

        let mut n_exclusions = 0;
        let mut n_pbs = 0;
        let p = hprof::enter("expand_resources");
        for (_token_idx, rc) in resource_constraints.iter_mut() {
            if rc.users.len() > rc.integrated {
                // We need to update the constraint.

                if rc.integrated != 0 {
                    // println!("WARNING: resource constraint users has been extended.");
                }

                rc.integrated = rc.users.len();

                if !rc.closed {
                    // TODO: make an extension point in the pseudo-boolean constraint for adding more usages later.
                }

                // println!(
                //     "Adding resource constraint for {}.{} with size {} capacity {:?}",
                //     timeline_names[states[tokens[*_token_idx].state].timeline],
                //     tokens[*_token_idx].value,
                //     rc.users.len(),
                //     rc.capacity
                // );

                // TASK-INDEXED RESOURCE CONSTRAINT

                // for i in 0..rc.users.len() {
                //     let j0 = if i > rc.integrated {
                //         0
                //     } else {
                //         i+1
                //     };

                //     for j in j0..rc.users.len() {

                //     }
                // }

                const USE_PAIRWISE_RESOURCE_CONSTRAINT: bool = true;

                if USE_PAIRWISE_RESOURCE_CONSTRAINT && rc.capacity.unwrap() == 1 {
                    // Special-case parwise exclusion, which is probably faster than
                    // the long pseudo-boolean constraint needed for capacity >=2

                    println!("Cap1 exclusion {}", rc.users.len());
                    for i in 0..rc.users.len() {
                        for j in (i + 1)..rc.users.len() {
                            let (link1, token1, amount1) = &rc.users[i];
                            let (link2, token2, amount2) = &rc.users[j];

                            assert!(*amount1 == 1);
                            assert!(*amount2 == 1);

                            let mut alts = vec![
                                Real::le(
                                    &states[tokens[*token1].state].end_time,
                                    &states[tokens[*token2].state].start_time,
                                ),
                                Real::le(
                                    &states[tokens[*token2].state].end_time,
                                    &states[tokens[*token1].state].start_time,
                                ),
                            ];

                            if let Some(link1) = link1 {
                                alts.push(link1.not());
                            }
                            if let Some(link2) = link2 {
                                alts.push(link2.not());
                            }

                            let alts_refs = alts.iter().collect::<Vec<_>>();
                            solver.assert(&Bool::or(&ctx, &alts_refs));
                            n_exclusions += 1;
                        }
                    }
                } else {
                    // println!("Cap >=2 PB");
                    for (link1, token1, _) in rc.users.iter() {
                        // println!("link1 const {:?}", link1);
                        let overlaps = rc
                            .users
                            .iter()
                            .map(|(link2, token2, amount2)| {
                                // println!("   link2 const {:?}", link2);
                                let overlap = Bool::and(
                                    &ctx,
                                    &[
                                        // &link1.clone().unwrap_or_else(|| Bool::from_bool(&ctx, true)),
                                        &link2.clone().unwrap_or_else(|| Bool::from_bool(&ctx, true)),
                                        &Real::lt(
                                            &states[tokens[*token1].state].start_time,
                                            &states[tokens[*token2].state].end_time,
                                        ),
                                        &Real::lt(
                                            &states[tokens[*token2].state].start_time,
                                            &states[tokens[*token1].state].end_time,
                                        ),
                                    ],
                                );

                                (overlap, *amount2)
                            })
                            .collect::<Vec<_>>();

                        let overlaps_refs = overlaps.iter().map(|(o, c)| (o, *c as i32)).collect::<Vec<_>>();

                        // println!(
                        //     "Adding resource constraint for {}.{} with size {} cap {}",
                        //     timeline_names[states[tokens[*_token_idx].state].timeline],
                        //     tokens[*_token_idx].value,
                        //     overlaps.len(),
                        //     rc.capacity.unwrap()
                        // );

                        let pb = Bool::pb_le(&ctx, &overlaps_refs, rc.capacity.unwrap() as i32);
                        if let Some(link1) = link1 {
                            solver.assert(&Bool::implies(link1, &pb));
                        } else {
                            solver.assert(&pb);
                        }
                        n_pbs += 1;
                    }
                }
            }
        }

        // Now we have refined the problem enough for a potential solution to come from solving the SMT.
        // Will call the SMT solver with a list of assumptions that negate all the extension literals.
        // Extensions are:
        //  - state reaches goal and doesn't transition from then
        //  - conditions choose from the set of possible causal links
        //  - possibly: resource constraint extension literals.

        drop(p);
        let p = hprof::enter("solve_smt");

        let expand_state_seq_lits: HashMap<Bool, usize> = timelines
            .iter()
            .map(|tl| *tl.states.last().unwrap())
            .map(|s_idx| (states[s_idx].activate_next.clone(), s_idx))
            .collect();

        let neg_expansions = expand_links_lits
            .keys()
            .chain(expand_goal_state_lits.keys())
            .chain(expand_state_seq_lits.keys())
            .map(|l| (Bool::not(l), l.clone()))
            .collect::<HashMap<_, _>>();

        // for (i, timeline) in timelines.iter().enumerate() {
        //     println!("Timeline {} has {} states", timeline_names[i], timeline.states.len());
        // }

        println!(
            "Solving with {} timelines {} states {} tokens {} conditions {} goal_exp {} link_exp {} pairexcl. {} pbs",
            timelines.len(),
            states.len(),
            tokens.len(),
            conds.len(),
            expand_goal_state_lits.len(),
            expand_links_lits.len(),
            n_exclusions,
            n_pbs,
        );

        // println!("{}", solver.to_string());
        // panic!();

        n_smt_calls += 1;
        let result = solver.check_assumptions(&neg_expansions.keys().cloned().collect::<Vec<_>>());
        drop(p);

        match result {
            z3::SatResult::Unsat => {
                let p = hprof::enter("unsat_core");
                let mut core = solver.get_unsat_core();
                if core.is_empty() {
                    return Err(SolverError::NoSolution);
                }

                let use_trim_core = true;
                let use_minimize_core = true;

                if use_trim_core {
                    crate::cores::trim_core(&mut core, &solver, |_| {});
                }

                if use_minimize_core {
                    crate::cores::minimize_core(&mut core, &solver, |_| {});
                }

                // core_sizes.push(core.len());
                println!("CORE SIZE #{}", core.len());
                for c in core {
                    if let Some(nc) = neg_expansions.get(&c) {
                        if let Some(timeline) = expand_goal_state_lits.get(nc) {
                            println!("Expand goals in timleine {}", problem.timelines[*timeline].name);
                            println!(
                                "  -expand GOALs for {}",
                                // problem.timelines[states[token.state].timeline].name, token.value, cond.cond_spec
                                timeline_names[*timeline]
                            );
                            // todo!()
                        } else if let Some(cond_idx) = expand_links_lits.get(nc).copied() {
                            let cond = &conds[cond_idx];
                            let token = &tokens[cond.token_idx];
                            println!(
                                "  -expand LINK {}.{} {:?}",
                                problem.timelines[states[token.state].timeline].name, token.value, cond.cond_spec
                            );

                            // TODO heuristically decide which and how many to expand.s
                            expand_links_queue.push((true, cond_idx));
                            // need_more_links_than = links.len();
                        } else if let Some(state_idx) = expand_state_seq_lits.get(nc).copied() {
                            let timeline = timeline_names[states[state_idx].timeline];
                            let values = states[state_idx]
                                .tokens
                                .iter()
                                .map(|t| tokens[*t].value)
                                .collect::<Vec<_>>();
                            println!(
                                "need to expand state because of MetBy condition cross-timeline {} state{} values{:?}",
                                timeline, state_idx, values
                            );

                            expand_n(
                                problem,
                                &ctx,
                                &solver,
                                states[state_idx].timeline,
                                &mut timelines,
                                &mut states,
                                &mut tokens,
                                1,
                            );
                        } else {
                            panic!("didn't find positive core lit");
                        }
                    } else {
                        panic!("didn't find negated core lit");
                    }
                }
            }

            z3::SatResult::Sat => {
                let p = hprof::enter("extract_solution");
                // println!("SAT after {} solver calls", n_smt_calls);
                let model = solver.get_model().unwrap();
                // println!("{}", model.to_string());

                let mut solution_tokens = Vec::new();
                for v in tokens.iter() {
                    let active = v
                        .active
                        .as_ref()
                        .map(|a| model.eval(a, true).unwrap().as_bool().unwrap())
                        .unwrap_or(true);

                    if !active {
                        // println!("token {} ({:?}) not active", v.value, v.active);
                        continue;
                    }

                    let start_time = from_z3_real(&model.eval(&states[v.state].start_time, true).unwrap());
                    let end_time = from_z3_real(&model.eval(&states[v.state].end_time, true).unwrap());

                    // println!("value {:?}", v.value);

                    solution_tokens.push(SolutionToken {
                        object_name: timeline_names[states[v.state].timeline].to_string(),
                        value: v.value.to_string(),
                        start_time,
                        end_time,
                    })
                }

                // for tl in 0..timelines.len() {
                //     println!("Timeline {}", timeline_names[tl]);
                //     for state in timelines[tl].states.iter().copied() {
                //         println!("  State #{}", state);
                //         for token in states[state].tokens.iter().copied() {
                //             let active = tokens[token]
                //                 .active
                //                 .as_ref()
                //                 .map(|a| model.eval(a, true).unwrap().as_bool().unwrap());

                //             println!("    Token {}: {:?}", tokens[token].value, active);
                //         }
                //     }
                // }

                // println!("SOLUTION {:#?}", timelines);

                return Ok(Solution {
                    tokens: solution_tokens,
                });
            }

            z3::SatResult::Unknown => {
                panic!("Z3 is undecided.")
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn expand_until<'a, 'z>(
    problem: &'a Problem,
    ctx: &'z z3::Context,
    solver: &z3::Solver,
    timeline_idx: usize,
    timelines: &mut Vec<Timeline<'z>>,
    states: &mut Vec<State<'z>>,
    tokens: &mut Vec<Token<'a, 'z>>,
    value: Option<&str>,
) -> bool {
    let n = if let Some(value) = value {
        assert!(!timelines[timeline_idx].states.is_empty());
        let prev_state = &states[*timelines[timeline_idx].states.last().unwrap()];
        let prev_values = prev_state.tokens.iter().map(|t| tokens[*t].value).collect::<Vec<_>>();

        if let Some(n) = distance_to(&problem.timelines[timeline_idx], &prev_values, value) {
            n
        } else {
            return false;
        }
    } else {
        1
    };

    assert!(n > 0);
    expand_n(problem, ctx, solver, timeline_idx, timelines, states, tokens, n);
    true
}

#[allow(clippy::too_many_arguments)]
fn expand_n<'a, 'z>(
    problem: &'a Problem,
    ctx: &'z z3::Context,
    solver: &z3::Solver,
    timeline_idx: usize,
    timelines: &mut Vec<Timeline<'z>>,
    states: &mut Vec<State<'z>>,
    tokens: &mut Vec<Token<'a, 'z>>,
    n: usize,
) {
    for _ in 0..n {
        let (state_seq, start_time, prev_values) =
            if let Some(prev_state_idx) = timelines[timeline_idx].states.last().copied() {
                let prev_state = &states[prev_state_idx];
                let prev_values = prev_state.tokens.iter().map(|t| tokens[*t].value).collect::<Vec<_>>();
                let seq = prev_state.state_seq + 1;

                (seq, prev_state.end_time.clone(), Some(prev_values))
            } else {
                (0, Real::fresh_const(ctx, "t"), None)
            };

        let end_time = Real::fresh_const(ctx, "t");

        let state_idx = states.len();
        let token_start_idx = tokens.len();
        let values = next_values_from(&problem.timelines[timeline_idx], prev_values.as_deref());

        // println!("adding tl:{} state:{} values{:?}", problem.timelines[timeline_idx].name, state_seq,values );

        let state_tokens = values
            .iter()
            .map(|value| {
                // let prev_unique = prev_values.is_none() || prev_values.as_ref().unwrap().len() == 1;
                // let active = if prev_unique && values.len() == 1 {
                //     None // only one chocie heree
                // } else {
                //     Some(Bool::fresh_const(ctx, "x"))
                // };

                let active = Some(Bool::fresh_const(ctx, "x"));

                Token {
                    active,
                    state: state_idx,
                    value,
                    fact: false,
                }
            })
            .collect::<Vec<_>>();

        if state_tokens.is_empty() {
            println!("No initial state for timeline {}", problem.timelines[timeline_idx].name);
            panic!();
        }

        // At most one state can be chosen.
        let am1 = state_tokens
            .iter()
            .filter_map(|t| t.active.as_ref().map(|b| (b, 1)))
            .collect::<Vec<_>>();
        if am1.len() > 1 {
            solver.assert(&Bool::pb_le(ctx, &am1, 1));
        }

        let tokens_active = state_tokens
            .iter()
            .map(|t| t.active.as_ref().unwrap())
            .collect::<Vec<_>>();
        let state_active = Bool::or(ctx, &tokens_active);

        if state_seq > 0 {
            // for state_token in state_tokens.iter() {
            //     // If a token is active, the previous state must also be active.
            //     let mut clause = Vec::new();
            //     if let Some(active) = state_token.active.as_ref() {
            //         clause.push(Bool::not(active));
            //     }

            //     // any in the previous state
            //     let prev_state_idx = timelines[timeline_idx].states[state_seq - 1];
            //     let mut any_const = false;
            //     for token in states[prev_state_idx].tokens.iter().copied() {
            //         if let Some(active) = tokens[token].active.as_ref() {
            //             clause.push(active.clone());
            //         } else {
            //             any_const = true;
            //         }
            //     }

            //     if !any_const {
            //         let clause_refs = clause.iter().collect::<Vec<_>>();
            //         solver.assert(&Bool::or(ctx, &clause_refs));
            //     }
            // }

            let prev_state_idx = timelines[timeline_idx].states[state_seq - 1];
            solver.assert(&Bool::implies(&state_active, &states[prev_state_idx].active))
        }

        let token_idxs = state_tokens
            .iter()
            .enumerate()
            .map(|(i, _)| token_start_idx + i)
            .collect::<Vec<_>>();

        tokens.extend(state_tokens);
        states.push(State {
            state_seq,
            tokens: token_idxs,
            start_time,
            end_time,
            timeline: timeline_idx,
            active: state_active,
            activate_next: Bool::fresh_const(ctx, "nxstate"),
        });
        timelines[timeline_idx].states.push(state_idx);
    }
}

fn next_values_from<'a>(timeline: &'a problem::Timeline, prev_values: Option<&[&'a str]>) -> HashSet<&'a str> {
    let mut next_values: HashSet<&str> = Default::default();

    for value_spec in timeline.values.iter() {
        if let Some(prev_values) = prev_values {
            // When we are looking for a next value from a previous one, if any of the
            // previous values are referred to as a transition condition, then the value is included.
            if prev_values.iter().any(|pv| {
                value_spec
                    .conditions
                    .iter()
                    .any(|c| c.is_timeline_transition_from(&timeline.name) == Some(pv))
            }) {
                next_values.insert(&value_spec.name);
            }
        } else {
            // If we are looking for an initial state, none of the conditions can be transitions conditions.
            if !value_spec
                .conditions
                .iter()
                .any(|c| c.is_timeline_transition_from(&timeline.name).is_some())
            {
                next_values.insert(&value_spec.name);
            }
        }
    }

    // Also, all the transition TO specific values are included.
    if let Some(prev_values) = prev_values {
        for prev_value in prev_values.iter() {
            if let Some(value_spec) = timeline.values.iter().find(|v| &v.name == prev_value) {
                for cond in value_spec.conditions.iter() {
                    if let Some(target) = cond.is_timeline_transition_to(&timeline.name) {
                        next_values.insert(target);
                    }
                }
            }
        }
    }

    // println!("Next values from {} {:?} {:?}", timeline.name, prev_values, next_values);

    next_values
}

fn can_expand(timeline: &problem::Timeline, start_values: &[&str], goal_value: &str) -> bool {
    distance_to(timeline, start_values, goal_value).is_some()
}

fn distance_to(timeline: &problem::Timeline, start_values: &[&str], goal_value: &str) -> Option<usize> {
    let mut visited_values = HashSet::new();
    let mut current_values = start_values.iter().copied().collect::<HashSet<_>>();

    let mut steps = 1;
    loop {
        let mut next_values = HashSet::new();
        for next in next_values_from(timeline, Some(&current_values.iter().copied().collect::<Vec<_>>())) {
            if goal_value == next {
                return Some(steps);
            }

            if visited_values.insert(next) {
                next_values.insert(next);
            }
        }

        if next_values.is_empty() {
            return None;
        }

        current_values = next_values;
        steps += 1;
    }
}

#[derive(Default)]
struct ResourceConstraint<'z3> {
    capacity: Option<u32>,
    users: Vec<(Option<Bool<'z3>>, usize, u32)>,
    integrated: usize,
    closed: bool,
}
