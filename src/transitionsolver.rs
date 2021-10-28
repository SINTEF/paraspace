use std::collections::HashMap;

use z3::ast::{Bool, Real};

use crate::{
    problem::{Problem, Solution},
    SolverError,
};

pub fn solve(problem: &Problem) -> Result<Solution, SolverError> {
    let z3_config = z3::Config::new();
    let ctx = z3::Context::new(&z3_config);
    let solver = z3::Solver::new(&ctx);

    // A state is a choice between several possible tokens
    // in the sequence of values that make up a timeline.
    struct State<'a, 'z> {
        constant: bool,
        start_time: Real<'z>,
        end_time: Real<'z>,
        values: Vec<(&'a str, Bool<'z>)>,
    }

    struct Timeline {
        states: Vec<usize>,
        constant: bool,
    }

    let mut timelines = problem
        .timelines
        .iter()
        .map(|t| Timeline {
            states: Vec::new(),
            constant: false,
        })
        .collect::<Vec<_>>();

    let mut timelines_queue = 0;
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

    // The goal states need to be the last states.
    // The facts need to be the first states.

    for const_token in problem.tokens.iter() {
        if !timelines_by_name.contains_key(const_token.timeline_name.as_str()) {
            timelines_by_name.insert(const_token.timeline_name.as_str(), timelines.len());
            timelines.push(Timeline {
                states: Vec::new(),
                constant: true,
            });
        }

        match const_token.const_time {
            crate::problem::TokenTime::Fact(_, _) => todo!(),
            crate::problem::TokenTime::Goal => {
                let state_idx = states.len();
                states.push(State {
                    constant: true,
                    end_time: Real::fresh_const(&ctx, "t"),
                    start_time: Real::fresh_const(&ctx, "t"),
                    values: vec![(const_token.value.as_str(), Bool::from_bool(&ctx, true))],
                });
                timelines[timelines_by_name[const_token.timeline_name.as_str()]]
                    .states
                    .push(state_idx);
            }
        }
    }

    todo!()
}
