use z3::ast::{Bool, Real};

use crate::problem::*;
use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Debug)]
pub enum SolverError {
    NoSolution,
    MultipleLastValues { timeline: String },
    GoalValueDurationLimit,
}

pub fn solve(problem: &Problem) -> Result<(), SolverError> {
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
        let value_idx = values.len();
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
            end_time :None,
            condition: None, // unconditional
        });
    }

    struct ExpandLinkIdx(usize);
    let mut expansions: HashMap<Bool, ExpandLinkIdx> = HashMap::new();

    loop {
        // Expand the graph and try to solve
        while value_link_queue < values.len() {
            // add all the links for the value
            let value = &values[value_link_queue];
            value_link_queue += 1;

            
            let state = problem.timelines[value.timeline_idx]
                .states
                .iter()
                .find(|s| s.name == value.value)
                .unwrap();

            for condition in state.conditions.iter() {
                match condition {
                    Condition::UseResource(obj, amount) => {}
                    Condition::TransitionFrom(prev_value) => {
                    }
                    Condition::During(tl, v) => {
                       links.push(0); 
                    }
                    Condition::MetBy(tl, v) => todo!(),
                };
            }
        }

        let assumptions = expansions.keys().cloned().collect::<Vec<_>>();
        let result = solver.check_assumptions(&assumptions);
        match result {
            z3::SatResult::Unsat => {
                let core = solver.get_unsat_core();
                if core.is_empty() {
                    return Err(SolverError::NoSolution);
                }

                for c in core {
                    let ExpandLinkIdx(link_idx) = expansions.remove(&c).unwrap();
                    todo!("Expand link_idx");
                }
            }

            z3::SatResult::Sat => {
                return Ok(());
            }

            z3::SatResult::Unknown => {
                panic!("Z3 is undecided.")
            }
        }
    }
}

struct Value<'a, 'z3> {
    start_time :Real<'z3>,
    end_time :Option<Real<'z3>>,
    timeline_idx: usize,
    value: &'a str,
    condition: Option<Bool<'z3>>,
}

struct Link {
    timeline_idx :usize,
}

// struct ValueGraph<'a> {
//     timeline: &'a Timeline,
//     last_value: Option<()>,
// }

// impl<'a> ValueGraph<'a> {
//     pub fn new(timeline: &'a Timeline) -> Self {
//         Self {
//             timeline,
//             last_value: None,
//         }
//     }

//     pub fn set_last_value(&mut self, value: &'a str) -> Result<(), SolverError> {
//         if self.last_value.is_some() {
//             return Err(SolverError::MultipleLastValues {
//                 timeline: self.timeline.name.as_ref().cloned().unwrap_or_else(String::new),
//             });
//         }
//         self.last_value = Some(());

//         Ok(())
//     }
// }
