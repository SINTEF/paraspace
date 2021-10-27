use z3::ast::Ast;
use z3::ast::{Bool, Real};

use crate::{multiplicity::multiplicity_one, problem::*};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum SolverError {
    NoSolution,
    GoalValueDurationLimit,
    GoalStateMissing,
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

    let groups_by_name = problem
        .groups
        .iter()
        .map(|g| (g.name.as_str(), &g.members))
        .collect::<HashMap<_, _>>();

    let mut tokens = Vec::new();
    let mut tokens_by_name_and_generation: HashMap<(&str, &str), Vec<(usize, usize)>> = HashMap::new();
    let mut token_queue = 0;

    let mut links = Vec::new();
    let mut link_queue = 0;

    let mut resource_constraints: HashMap<usize, ResourceConstraint> = Default::default(); // token to resourceconstraint
    
    let mut expand_links_queue :Vec<usize> = Vec::new();
    let mut expand_links_lits: HashMap<Bool, usize> = HashMap::new();
    let mut need_more_links_than = 0;

    // Pre-specified tokens: facts and goals.
    for token_spec in problem.tokens.iter() {
        let token_idx = tokens.len();
        let (fact, start_time, end_time) = match token_spec.const_time {
            TokenTime::Fact(start, end) => (
                true,
                Some(
                    start
                        .map(|t| Real::from_real(&ctx, t as i32, 1))
                        .unwrap_or_else(|| Real::fresh_const(&ctx, &format!("t_{}_{}_start_{}",token_spec.timeline_name, token_spec.value, token_idx))),
                ),
                Some(
                    end.map(|t| Real::from_real(&ctx, t as i32, 1))
                        .unwrap_or_else(|| Real::fresh_const(&ctx, &format!("t_{}_{}_end_{}",token_spec.timeline_name, token_spec.value, token_idx))),
                ),
            ),
            TokenTime::Goal => (false, Some(Real::fresh_const(&ctx, &format!("t_{}_{}_gstart_{}",token_spec.timeline_name, token_spec.value, token_idx))), None),
        };
        if !fact {
            let timeline_idx = *timelines_by_name
                .get(token_spec.timeline_name.as_str())
                .ok_or(SolverError::GoalStateMissing)?;

            let state = problem.timelines[timeline_idx]
                .states
                .iter()
                .find(|s| s.name == token_spec.value)
                .ok_or(SolverError::GoalStateMissing)?;

            if state.duration.1.is_some() {
                return Err(SolverError::GoalValueDurationLimit);
            }
        }

        // println!("Adding token {} {} {}", if fact { "fact"} else {"goal"}, token_spec.timeline_name, token_spec.value);

        tokens.push(Token {
            timeline_name: &token_spec.timeline_name,
            start_time,
            value: token_spec.value.as_str(),
            end_time,
            active: Bool::from_bool(&ctx, true), // unconditional
            fact,
        });
        resource_constraints.entry(token_idx).or_default().capacity = Some(token_spec.capacity);

        tokens_by_name_and_generation
            .entry((&token_spec.timeline_name, &token_spec.value))
            .or_default()
            .push((0, token_idx));
    }

    // println!("Tokens: {:?}", tokens_by_name);

    loop {
        // Expand the graph and try to solve
        while token_queue < tokens.len() || link_queue < links.len() {
            while token_queue < tokens.len() {
                // add all the links for the value
                let token_idx = token_queue;
                let token = &tokens[token_idx];
                token_queue += 1;

                // Process newly added token:
                //  - add its internal constraints (duration limits), and
                //  - add its preconditions (links) to be processed.

                // println!("Expanding graph for {}", token.value);

                // Facts don't need causal links or duration constraints.
                if token.fact {
                    solver.assert(&Real::le(
                        &Real::add(&ctx, &[token.start_time.as_ref().unwrap(), &Real::from_real(&ctx, 1, 1)]),
                        token.end_time.as_ref().unwrap(),
                    ));
                    continue;
                }

                // let timeline_name = problem.timelines[token.timeline_idx].name.as_str();
                let timeline_idx = timelines_by_name[token.timeline_name];
                if let Some(state) = problem.timelines[timeline_idx]
                    .states
                    .iter()
                    .find(|s| s.name == token.value)
                {
                    resource_constraints.entry(token_idx).or_default().capacity = Some(state.capacity);
                    // println!("Token {} {}", token.timeline_name, token.value);

                    if let Some(end_time) = token.end_time.as_ref() {
                        if let Some(start_time) = token.start_time.as_ref() {
                            solver.assert(&Real::le(
                                &Real::add(&ctx, &[start_time, &Real::from_real(&ctx, state.duration.0 as i32, 1)]),
                                end_time,
                            ));

                            if let Some(max_dur) = state.duration.1 {
                                solver.assert(&Real::ge(
                                    &Real::add(&ctx, &[start_time, &Real::from_real(&ctx, max_dur as i32, 1)]),
                                    end_time,
                                ));
                            }
                        }
                    }

                    for condition in state.conditions.iter() {
                        // println!("  -cond {} {:?}", token_idx, condition);
                        links.push(Link {
                            token_idx,
                            linkspec: condition,
                            alternatives: Vec::new(),
                            alternatives_extension: None,
                            token_queue: 0,
                        });
                    }
                } else {
                    let disable = Bool::not(&token.active);
                    // println!("This state doesn't exist. Asserting {:?}", disable);
                    solver.assert(&disable);
                }
            }
            while !expand_links_queue.is_empty() || link_queue < links.len() {
                
                let link_idx = if link_queue < links.len() {
                    let link_idx = link_queue;
                    link_queue += 1;
                    link_idx
                } else {
                    println!("Expanding link from expand queue.");
                    expand_links_queue.pop().unwrap()
                };

                let link = &links[link_idx];
                // let token = &tokens[link.token_idx];
                // let timeline_name = problem.timelines[token.timeline_idx].name.as_str();

                // println!("Expanding link for {}", token.value);

                // All eligible objects for linking to.
                let objects: Vec<&str> = match &link.linkspec.object {
                    ObjectSet::Group(c) => groups_by_name
                        .get(c.as_str())
                        .iter()
                        .flat_map(|c| c.iter().map(String::as_str))
                        .collect::<Vec<_>>(),
                    ObjectSet::Object(n) => {
                        vec![n.as_str()]
                    }
                };

                let mut new_alternatives = Vec::new();

                let mut candidate_tokens = Vec::new();
                for obj in objects.iter() {
                    if let Some(token_ref_list) = tokens_by_name_and_generation.get(&(obj, &link.linkspec.value)) {
                        candidate_tokens.extend(token_ref_list.iter().filter_map(|(gen,tok)| *tok));
                    }
                }

                let total_multiplicity = objects
                    .iter()
                    .map(|o| {
                        if multiplicity_one.contains(&(o, &link.linkspec.value)) {
                            1
                        } else {
                            2
                        }
                    })
                    .sum::<usize>();

                if total_multiplicity >= 2 {
                    let expand_lit = Bool::fresh_const(&ctx, "exp");
                    expand_links_lits.insert(Bool::not(&expand_lit), link_idx);
                    links[link_idx].alternatives_extension = Some(expand_lit.clone());
                    new_alternatives.push(expand_lit);
                } else {
                    // println!(
                    //     "NO MORE ALTERNATIVES FOR {} {}",
                    //     problem.timelines[token.timeline_idx].name, token.value
                    // );
                }

                let link = &links[link_idx];
                if candidate_tokens.is_empty() {
                    // Select a object at random
                    // TODO make a heuristic for this

                    if let Some(obj_name) = objects.get(0) {
                        let new_token_idx = tokens.len();
                        //     let token_active = ;

                        tokens.push(Token {
                            timeline_name: obj_name,
                            active: Bool::fresh_const(&ctx, "pre"),
                            fact: false,
                            value: &link.linkspec.value,
                            start_time: Some(Real::fresh_const(&ctx, &format!("t_{}_{}_start_{}",obj_name, link.linkspec.value, new_token_idx))),
                            end_time: Some(Real::fresh_const(&ctx, &format!("t_{}_{}_start_{}",obj_name, link.linkspec.value, new_token_idx))),
                        });

                        candidate_tokens.push(new_token_idx);
                        tokens_by_name_and_generation
                            .entry((obj_name, &link.linkspec.value))
                            .or_default()
                            .push(new_token_idx);
                    }
                }
                let token = &tokens[link.token_idx];

                for token_idx in candidate_tokens.iter().copied() {
                    println!("linking value {}.{} {}.{}\n --{:?}", tokens[token_idx].timeline_name, tokens[token_idx].value, token.timeline_name, token.value, link.linkspec);

                    // Represents the usage of the causal link.
                    let choose_link = Bool::fresh_const(&ctx, "cl");

                    let temporal_rel = match link.linkspec.temporal_relationship {
                        TemporalRelationship::Meet => vec![Real::_eq(
                            tokens[token_idx].end_time.as_ref().unwrap(),
                            token.start_time.as_ref().unwrap(),
                        )],
                        TemporalRelationship::Cover => vec![
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

                    if link.linkspec.amount > 0 {
                        println!("Link has amount {:?}", link.linkspec);
                        // Add resource constraint for this token.
                        resource_constraints.entry(token_idx).or_default().users.push((
                            choose_link.clone(),
                            link.token_idx,
                            link.linkspec.amount,
                        ));
                    }

                    // The choose_link boolean implies all the condntions.
                    for cond in temporal_rel.iter().chain(std::iter::once(&tokens[token_idx].active)) {
                        solver.assert(&Bool::implies(&choose_link, cond));
                    }

                    new_alternatives.push(choose_link);
                }

                let alternatives_refs = new_alternatives.iter().collect::<Vec<_>>();
                // println!(
                //     "TOKEN LINKS for {}.{}[{}] has {} alternatives",
                //     problem.timelines[tokens[link.token_idx].timeline_idx].name,
                //     tokens[link.token_idx].value,
                //     link.token_idx,
                //     alternatives.len()
                // );
                solver.assert(&Bool::implies(
                    &tokens[link.token_idx].active,
                    &Bool::or(&ctx, &alternatives_refs),
                ));
            }
        }

        // for (obj, rc) in resource_constraints.iter() {
        //     // We don't yet support name-based and class-based resource references at the same time,
        //     // so check that the spec doesn't do that.
        //     if let ObjectRef::Named(name) = obj {
        //         let timeline = &problem.timelines[timelines_by_name[name.as_str()]];
        //         if resource_constraints
        //             .iter()
        //             .any(|(other_objref, _)| *other_objref == &ObjectRef::AnyOfClass(timeline.class.clone()))
        //         {
        //             return Err(SolverError::UnsupportedInput);
        //         }
        //     }
        // }

        // Need to check all the resource constraints to see if they need to be "integrated".
        // The resource constraints cannot generate new tokens or links, so this can be done in a separate non-loop here.
        for (_token_idx, rc) in resource_constraints.iter_mut() {
            if rc.users.len() > rc.integrated {
                // We need to update the constraint.

                rc.integrated = rc.users.len();

                if !rc.closed {
                    // TODO: make an extension point in the pseudo-boolean constraint for adding more usages later.
                }

                // TASK-INDEXED RESOURCE CONSTRAINT
                for (link1, token1, _) in rc.users.iter() {
                    let overlaps = rc
                        .users
                        .iter()
                        .map(|(link2, token2, amount2)| {
                            let overlap = Bool::and(
                                &ctx,
                                &[
                                    link1,
                                    link2,
                                    &Real::lt(
                                        tokens[*token1].start_time.as_ref().unwrap(),
                                        tokens[*token2].end_time.as_ref().unwrap(),
                                    ),
                                    &Real::lt(
                                        tokens[*token2].start_time.as_ref().unwrap(),
                                        tokens[*token1].end_time.as_ref().unwrap(),
                                    ),
                                ],
                            );

                            (overlap, *amount2)
                        })
                        .collect::<Vec<_>>();

                    let overlaps_refs = overlaps.iter().map(|(o, c)| (o, *c as i32)).collect::<Vec<_>>();

                    println!("Adding resource constraint for {}.{} with size {}", tokens[*_token_idx].timeline_name, tokens[*_token_idx].value, overlaps.len());
                    solver.assert(&Bool::pb_le(&ctx, &overlaps_refs, rc.capacity.unwrap() as i32));
                }
            }
        }

        if need_more_links_than > 0 && need_more_links_than == links.len() {
            // TODO this is not complete if we don't expand ALL of the core below. (but we do expand all, for now.)
            println!("Didn't expand any links!");
            return Err(SolverError::NoSolution);
        }

        let assumptions = expand_links_lits.keys().cloned().collect::<Vec<_>>();
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

                // println!("CORE {:?}", core);
                for c in core {
                    let link_idx = expand_links_lits.remove(&c).unwrap();
                    let link = &links[link_idx];
                    let token = &tokens[link.token_idx];
                    // println!("  -expand {}.{} {:?}", token.timeline_name, token.value, link.linkspec);
                    
                    // TODO heuristically decide which and how many to expand.s
                    expand_links_queue.push(link_idx);
                    need_more_links_than = links.len();
                }
            }

            z3::SatResult::Sat => {
                let model = solver.get_model().unwrap();

                let mut solution_tokens = Vec::new();
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

                    solution_tokens.push(SolutionToken {
                        object_name: v.timeline_name.to_string(),
                        value: v.value.to_string(),
                        start_time,
                        end_time,
                    })
                }

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

fn from_z3_real(real: &Real) -> f32 {
    let (num, den) = real.as_real().unwrap();
    num as f32 / den as f32
}

struct Link<'a, 'z3> {
    token_idx: usize,
    linkspec: &'a Condition,
    token_queue: usize,
    alternatives: Vec<(usize, Bool<'z3>)>,
    alternatives_extension: Option<Bool<'z3>>,
}

struct Token<'a, 'z3> {
    start_time: Option<Real<'z3>>,
    end_time: Option<Real<'z3>>,
    timeline_name: &'a str,
    value: &'a str,
    active: Bool<'z3>,
    fact: bool,
}

#[derive(Default)]
struct ResourceConstraint<'z3> {
    capacity: Option<u32>,
    users: Vec<(Bool<'z3>, usize, u32)>,
    integrated: usize,
    closed: bool,
}
