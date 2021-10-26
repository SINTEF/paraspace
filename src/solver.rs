use z3::ast::Ast;
use z3::ast::{Bool, Real};

use crate::{multiplicity::multiplicity_one, problem::*};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum SolverError {
    NoSolution,
    GoalValueDurationLimit,
}

pub enum LinkCondition<'a> {
    Resource(&'a ObjectRef, usize),
    Temporal(TemporalType, Result<&'a ObjectRef, &'a str>, &'a str),
}

pub fn convert_condition<'a>(this: &'a str, cond: &'a Condition) -> LinkCondition<'a> {
    match cond {
        Condition::UseResource(o, a) => LinkCondition::Resource(o, *a),
        Condition::TransitionFrom(v) => LinkCondition::Temporal(TemporalType::Meet, Err(this), v),
        Condition::During(o, x) => LinkCondition::Temporal(TemporalType::Cover, Ok(o), x),
        Condition::MetBy(o, x) => LinkCondition::Temporal(TemporalType::Meet, Ok(o), x),
    }
}

pub enum TemporalType {
    Meet,
    Cover,
}

pub fn solve(problem: &Problem) -> Result<Solution, SolverError> {
    let z3_config = z3::Config::new();
    let ctx = z3::Context::new(&z3_config);
    let solver = z3::Solver::new(&ctx);

    let multiplicity_one = multiplicity_one(problem);

    let timelines_by_name = problem
        .timelines
        .iter()
        .enumerate()
        .map(|(i, t)| (t.name.as_str(), i))
        .collect::<HashMap<_, _>>();

    let _resoures_by_name = problem
        .resources
        .iter()
        .enumerate()
        .map(|(i, r)| (r.name.as_str(), i))
        .collect::<HashMap<_, _>>();

    // timelines by class
    // resources by class

    // let mut value_graphs = problem.timelines.iter().map(ValueGraph::new).collect::<Vec<_>>();

    let mut tokens = Vec::new();
    let mut token_queue = 0;
    let mut links = Vec::new();
    let mut link_queue = 0;
    let mut values_by_name: HashMap<(&str, &str), Vec<usize>> = HashMap::new();

    for goal in problem.goals.iter() {
        let timeline_idx = timelines_by_name[goal.timeline_name.as_str()];
        let state = problem.timelines[timeline_idx]
            .states
            .iter()
            .find(|s| s.name == goal.value)
            .unwrap();

        if state.duration.1.is_some() {
            return Err(SolverError::GoalValueDurationLimit);
        }

        let value_idx = tokens.len();
        tokens.push(Token {
            timeline_idx,
            start_time: Some(Real::fresh_const(&ctx, "t")),
            value: goal.value.as_str(),
            end_time: None,
            active: Bool::from_bool(&ctx, true), // unconditional
            fixed: Some(FixedValueType::Goal),
        });

        values_by_name
            .entry((&goal.timeline_name, &goal.value))
            .or_default()
            .push(value_idx);
    }

    for fact in problem.facts.iter() {
        let value_idx = tokens.len();
        let timeline_idx = timelines_by_name[fact.timeline_name.as_str()];
        tokens.push(Token {
            timeline_idx,
            start_time: None,
            value: fact.value.as_str(),
            end_time: Some(Real::fresh_const(&ctx, "t")),
            active: Bool::from_bool(&ctx, true), // unconditional
            fixed: Some(FixedValueType::Fact),
        });

        values_by_name
            .entry((&fact.timeline_name, &fact.value))
            .or_default()
            .push(value_idx);
    }

    let mut expand_links: HashMap<Bool, usize> = HashMap::new();

    loop {
        // Expand the graph and try to solve
        while token_queue < tokens.len() || link_queue < links.len() {
            while token_queue < tokens.len() {
                // add all the links for the value
                let token_idx = token_queue;
                let token = &tokens[token_idx];
                token_queue += 1;

                // println!("Expanding graph for {}", token.value);

                // Facts don't need causal links.
                if !matches!(token.fixed, Some(FixedValueType::Fact)) {
                    if let Some(state) = problem.timelines[token.timeline_idx]
                        .states
                        .iter()
                        .find(|s| s.name == token.value)
                    {
                        if let Some(end_time) = token.end_time.as_ref() {
                            if let Some(start_time) = token.start_time.as_ref() {
                                solver.assert(&Real::le(
                                    &Real::add(&ctx, &[start_time, &Real::from_real(&ctx, state.duration.0 as i32, 1)]),
                                    end_time,
                                ));
                                // println!(
                                //     "TOKEN {}.{} {:?}",
                                //     problem.timelines[token.timeline_idx].name, token.value, state.duration
                                // );

                                if let Some(max_dur) = state.duration.1 {
                                    solver.assert(&Real::ge(
                                        &Real::add(&ctx, &[start_time, &Real::from_real(&ctx, max_dur as i32, 1)]),
                                        end_time,
                                    ));
                                }
                            }
                        }

                        for condition in state.conditions.iter() {
                            links.push(Link { token_idx, condition });
                        }
                    } else {
                        // This state doesn't exist.
                        let disable = Bool::not(&token.active);
                        println!("This state doesn't exist. Asserting {:?}", disable);
                        solver.assert(&disable);
                    }
                }
            }
            while link_queue < links.len() {
                let link_idx = link_queue;
                link_queue += 1;

                let link = &links[link_idx];
                let token = &tokens[link.token_idx];
                let timeline_name = problem.timelines[token.timeline_idx].name.as_str();

                // println!("Expanding link for {}", token.value);
                match convert_condition(timeline_name, link.condition) {
                    LinkCondition::Resource(_, _) => {

                        // TODO: implement resources
                    }

                    LinkCondition::Temporal(temporal_relationship_type, objref, target_value) => {
                        // All eligible objects for linking to.
                        let objects: Vec<&str> = match &objref {
                            Ok(ObjectRef::AnyOfClass(c)) => problem
                                .timelines
                                .iter()
                                .filter_map(|t| (&t.class == c).then(|| t.name.as_str()))
                                .collect(),
                            Ok(ObjectRef::Named(n)) => {
                                vec![n.as_str()]
                            }
                            Err(n) => {
                                vec![n]
                            }
                        };

                        let mut alternatives = Vec::new();

                        let mut candidate_tokens = Vec::new();
                        for obj in objects.iter() {
                            if let Some(token_ref_list) = values_by_name.get(&(obj, target_value)) {
                                candidate_tokens.extend(token_ref_list.iter().copied());
                            }
                        }

                        let total_multiplicity = objects
                            .iter()
                            .map(|o| {
                                if multiplicity_one.contains(&(o, target_value)) {
                                    1
                                } else {
                                    2
                                }
                            })
                            .sum::<usize>();

                        if total_multiplicity >= 2 {
                            let expand_lit = Bool::fresh_const(&ctx, "exp");
                            expand_links.insert(expand_lit.clone(), link_idx);
                            alternatives.push(expand_lit);
                        } else {
                            // println!(
                            //     "NO MORE ALTERNATIVES FOR {} {}",
                            //     problem.timelines[token.timeline_idx].name, token.value
                            // );
                        }

                        if candidate_tokens.is_empty() {
                            // Select a object at random
                            // TODO make a heuristic for this

                            if let Some(obj_name) = objects.get(0) {
                                let new_token_idx = tokens.len();
                                //     let token_active = ;

                                tokens.push(Token {
                                    timeline_idx: timelines_by_name[obj_name],
                                    active: Bool::fresh_const(&ctx, "pre"),
                                    fixed: None,
                                    value: target_value,
                                    start_time: Some(Real::fresh_const(&ctx, "t")),
                                    end_time: Some(Real::fresh_const(&ctx, "t")),
                                });

                                candidate_tokens.push(new_token_idx);
                                values_by_name
                                    .entry((obj_name, target_value))
                                    .or_default()
                                    .push(new_token_idx);
                            }
                        }
                        let token = &tokens[link.token_idx];

                        for token_idx in candidate_tokens.iter().copied() {
                            // println!("linking value {} {}", value_idx, link.token_idx);

                            let temporal_rel = match temporal_relationship_type {
                                TemporalType::Meet => vec![Real::_eq(
                                    tokens[token_idx].end_time.as_ref().unwrap(),
                                    token.start_time.as_ref().unwrap(),
                                )],
                                TemporalType::Cover => vec![
                                    Real::le(
                                        tokens[token_idx].start_time.as_ref().unwrap(),
                                        token.start_time.as_ref().unwrap(),
                                    ),
                                    Real::le(
                                        token.end_time.as_ref().unwrap(),
                                        tokens[token_idx].end_time.as_ref().unwrap(),
                                    ),
                                ],
                            };

                            let mut conjunction = temporal_rel;
                            conjunction.push(tokens[token_idx].active.clone());
                            let conjunction_ref = conjunction.iter().collect::<Vec<_>>();

                            alternatives.push(Bool::and(&ctx, &conjunction_ref));
                        }

                        let alternatives_refs = alternatives.iter().collect::<Vec<_>>();

                        let cond = if alternatives_refs.len() == 1 {
                            alternatives[0].clone()
                        } else {
                            Bool::or(&ctx, &alternatives_refs)
                        };

                        // println!(
                        //     "TOKEN LINKS for {}.{}[{}] has {} alternatives",
                        //     problem.timelines[tokens[link.token_idx].timeline_idx].name,
                        //     tokens[link.token_idx].value,
                        //     link.token_idx,
                        //     alternatives.len()
                        // );
                        solver.assert(&Bool::implies(&tokens[link.token_idx].active, &cond));
                    }
                };
            }
        }

        let assumptions = expand_links.keys().map(|k| Bool::not(k)).collect::<Vec<_>>();
        println!("{}", solver);
        println!(
            "Solving with {} tokens {} causal links {} extension points",
            tokens.len(),
            links.len(),
            assumptions.len()
        );
        let result = solver.check_assumptions(&assumptions);
        match result {
            z3::SatResult::Unsat => {
                let core = solver.get_unsat_core();
                if core.is_empty() {
                    return Err(SolverError::NoSolution);
                }

                for c in core {
                    let _link_idx = expand_links.remove(&c).unwrap();
                    todo!("Expand link_idx");
                }
            }

            z3::SatResult::Sat => {
                let model = solver.get_model().unwrap();

                let mut timelines: Vec<SolutionTimeline> = problem
                    .timelines
                    .iter()
                    .map(|t| SolutionTimeline {
                        name: t.name.clone(),
                        class: t.class.clone(),
                        tokens: Vec::new(),
                    })
                    .collect();

                for v in tokens.iter() {
                    let start_time = v
                        .start_time
                        .as_ref()
                        .map(|t| from_z3_real(&model.eval(t, true).unwrap()))
                        .unwrap_or(f32::NEG_INFINITY);
                    let end_time = v
                        .end_time
                        .as_ref()
                        .map(|t| from_z3_real(&model.eval(t, true).unwrap()))
                        .unwrap_or(f32::INFINITY);

                    timelines[v.timeline_idx].tokens.push(SolutionToken {
                        value: v.value.to_string(),
                        start_time,
                        end_time,
                    })
                }

                println!("SOLUTION {:#?}", timelines);

                return Ok(Solution { timelines });
            }

            z3::SatResult::Unknown => {
                panic!("Z3 is undecided.")
            }
        }
    }
}

fn from_z3_real(real: &Real) -> f32 {
    let (num, den) = real.as_real().unwrap();
    num as f32 / den as f32
}

struct Link<'a> {
    token_idx: usize,
    condition: &'a Condition,
}

struct Token<'a, 'z3> {
    start_time: Option<Real<'z3>>,
    end_time: Option<Real<'z3>>,
    timeline_idx: usize,
    value: &'a str,
    active: Bool<'z3>,
    fixed: Option<FixedValueType>,
}

enum FixedValueType {
    Goal,
    Fact,
}
