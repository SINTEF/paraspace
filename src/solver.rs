use z3::ast::{Bool, Real};

use crate::problem::*;
use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Debug)]
pub enum SolverError {
    NoSolution,
    MultipleLastValues { timeline: String },
    GoalValueDurationLimit,
}

pub fn solve(problem: &Problem) -> Result<Solution, SolverError> {
    let z3_config = z3::Config::new();
    let ctx = z3::Context::new(&z3_config);
    let solver = z3::Solver::new(&ctx);

    let timelines_by_name = problem
        .timelines
        .iter()
        .enumerate()
        .filter_map(|(i, t)| t.name.as_ref().map(|n| (n.as_str(), i)))
        .collect::<HashMap<_, _>>();

    let resoures_by_name = problem
        .resources
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.name.as_ref().map(|n| (n.as_str(), i)))
        .collect::<HashMap<_, _>>();

    // timelines by class
    // resources by class

    // let mut value_graphs = problem.timelines.iter().map(ValueGraph::new).collect::<Vec<_>>();

    let mut values = Vec::new();
    let mut value_link_queue = 0;
    let mut links = Vec::new();
    let mut link_queue = 0;

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

        values.push(Value {
            timeline_idx,
            start_time: Real::fresh_const(&ctx, "t"),
            value: goal.value.as_str(),
            end_time: None,
            prefix: None, // unconditional
        });
    }

    let mut expand_links: HashMap<Bool, usize> = HashMap::new();

    loop {
        // Expand the graph and try to solve
        while value_link_queue < values.len() || link_queue < links.len() {
            while value_link_queue < values.len() {
                // add all the links for the value
                let value = &values[value_link_queue];

                let state = problem.timelines[value.timeline_idx]
                    .states
                    .iter()
                    .find(|s| s.name == value.value)
                    .unwrap();

                for condition in state.conditions.iter() {
                    links.push(Link {
                        value_idx: value_link_queue,
                        condition,
                    });
                }

                value_link_queue += 1;
            }
            while link_queue < links.len() {
                let link = &links[link_queue];
                let value = &values[link.value_idx];

                match link.condition {
                    Condition::UseResource(_, _) => todo!(),
                    Condition::TransitionFrom(prev) => {
                        let prefix = value.prefix.is_some().then(|| Bool::fresh_const(&ctx, "pre"));

                        let state = problem.timelines[value.timeline_idx]
                            .states
                            .iter()
                            .find(|s| s.name == value.value)
                            .unwrap();

                        let new_value = Value {
                            end_time: Some(value.start_time.clone()),
                            prefix,
                            start_time: Real::fresh_const(&ctx, "t"),
                            timeline_idx: value.timeline_idx,
                            value: prev,
                        };

                        let new_state = problem.timelines[new_value.timeline_idx]
                            .states
                            .iter()
                            .find(|s| s.name == new_value.value)
                            .unwrap();

                        solver.assert(&Real::lt(
                            &Real::add(
                                &ctx,
                                &[
                                    &new_value.start_time,
                                    &Real::from_real(&ctx, new_state.duration.0 as i32, 1),
                                ],
                            ),
                            &value.start_time,
                        ));
                        println!(
                            "{}.{} {:?}",
                            problem.timelines[value.timeline_idx].name.as_ref().unwrap(),
                            new_value.value,
                            state.duration
                        );

                        if let Some(max_dur) = new_state.duration.1 {
                            solver.assert(&Real::gt(
                                &Real::add(&ctx, &[&new_value.start_time, &Real::from_real(&ctx, max_dur as i32, 1)]),
                                &value.start_time,
                            ));
                        }

                        values.push(new_value);
                    }
                    Condition::During(_, _) => todo!(),
                    Condition::MetBy(_, _) => todo!(),
                };

                link_queue += 1;
            }
        }

        let assumptions = expand_links.keys().cloned().collect::<Vec<_>>();
        let result = solver.check_assumptions(&assumptions);
        match result {
            z3::SatResult::Unsat => {
                let core = solver.get_unsat_core();
                if core.is_empty() {
                    return Err(SolverError::NoSolution);
                }

                for c in core {
                    let link_idx = expand_links.remove(&c).unwrap();
                    todo!("Expand link_idx");
                }
            }

            z3::SatResult::Sat => {
                let model = solver.get_model().unwrap();

                let mut timelines: Vec<SolutionTimeline> = problem
                    .timelines
                    .iter()
                    .enumerate()
                    .map(|(i, t)| SolutionTimeline {
                        name: t.name.as_ref().cloned().unwrap_or_else(|| format!("{}_{}", t.class, i)),
                        class: t.class.clone(),
                        tokens: Vec::new(),
                    })
                    .collect();

                for v in values.iter() {
                    let start_time = from_z3_real(&model.eval(&v.start_time, true).unwrap());
                    let end_time = v
                        .end_time
                        .as_ref()
                        .map(|t| from_z3_real(&model.eval(t, true).unwrap()))
                        .unwrap_or(f32::INFINITY);

                    timelines[v.timeline_idx].tokens.push(Token {
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
    value_idx: usize,
    condition: &'a Condition,
}

struct Value<'a, 'z3> {
    start_time: Real<'z3>,
    end_time: Option<Real<'z3>>,
    timeline_idx: usize,
    value: &'a str,
    prefix: Option<Bool<'z3>>,
}
