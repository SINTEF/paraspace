use crate::{SolverError, from_z3_real};
use z3::ast::Ast;
use z3::ast::{Bool, Real};

use crate::{multiplicity::multiplicity_one, problem::*};
use std::collections::HashMap;



pub fn solve(problem: &Problem) -> Result<Solution, SolverError> {
    println!("Starting pure-token-based solver.");
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
    let mut tokens_by_name: HashMap<(&str, &str), Vec<usize>> = HashMap::new();
    let mut token_queue = 0;

    let mut links: Vec<Link> = Vec::new();
    let mut link_queue = 0;
    let mut bakes = Vec::new();

    let mut resource_constraints: HashMap<usize, ResourceConstraint> = Default::default(); // token to resourceconstraint

    let mut expand_links_queue: Vec<(bool, usize)> = Vec::new();
    let mut expand_links_lits: HashMap<Bool, usize> = HashMap::new();
    let mut need_more_links_than = 0;

    // Pre-specified tokens: facts and goals.
    for token_spec in problem.tokens.iter() {
        let token_idx = tokens.len();
        let (fact, start_time, end_time) = match token_spec.const_time {
            TokenTime::Fact(start, end) => (
                true,
                Some(start.map(|t| Real::from_real(&ctx, t as i32, 1)).unwrap_or_else(|| {
                    Real::fresh_const(
                        &ctx,
                        &format!(
                            "t_{}_{}_start_{}",
                            token_spec.timeline_name, token_spec.value, token_idx
                        ),
                    )
                })),
                Some(end.map(|t| Real::from_real(&ctx, t as i32, 1)).unwrap_or_else(|| {
                    Real::fresh_const(
                        &ctx,
                        &format!("t_{}_{}_end_{}", token_spec.timeline_name, token_spec.value, token_idx),
                    )
                })),
            ),
            TokenTime::Goal => (
                false,
                Some(Real::fresh_const(
                    &ctx,
                    &format!(
                        "t_{}_{}_gstart_{}",
                        token_spec.timeline_name, token_spec.value, token_idx
                    ),
                )),
                None,
            ),
        };
        if !fact {
            let timeline_idx = *timelines_by_name
                .get(token_spec.timeline_name.as_str())
                .ok_or(SolverError::GoalStateMissing)?;

            let state = problem.timelines[timeline_idx]
                .values
                .iter()
                .find(|s| s.name == token_spec.value)
                .ok_or(SolverError::GoalStateMissing)?;

            if state.duration.1.is_some() {
                return Err(SolverError::GoalValueDurationLimit);
            }
        }

        // println!(
        //     "Adding token[{}] {} {} {}",
        //     if fact { "fact" } else { "goal" },
        //     tokens.len(),
        //     token_spec.timeline_name,
        //     token_spec.value
        // );

        tokens.push(Token {
            timeline_name: &token_spec.timeline_name,
            start_time,
            value: token_spec.value.as_str(),
            end_time,
            active: Bool::from_bool(&ctx, true), // unconditional
            fact,
        });
        resource_constraints.entry(token_idx).or_default().capacity = Some(token_spec.capacity);

        tokens_by_name
            .entry((&token_spec.timeline_name, &token_spec.value))
            .or_default()
            .push(token_idx);
    }

    // println!("Tokens: {:?}", tokens_by_name);

    let mut core_sizes = Vec::new();

    loop {
        // Expand the graph and try to solve
        while token_queue < tokens.len() || !expand_links_queue.is_empty() || link_queue < links.len() {
            while token_queue < tokens.len() {
                // add all the links for the value
                let token_idx = token_queue;
                let token = &tokens[token_idx];
                token_queue += 1;
                // println!("token idx {}", token_idx);

                if token.value == "Baking" {
                    bakes.push(token_idx);
                }

                // Process newly added token:
                //  - add its internal constraints (duration limits), and
                //  - add its preconditions (links) to be processed.

                // println!("Expanding graph for {}", token.value);

                // Facts don't need causal links or duration constraints.
                if token.fact {
                    solver.assert(&Real::le(
                        &Real::add(
                            &ctx,
                            &[token.start_time.as_ref().unwrap(), &Real::from_real(&ctx, 1, 1)],
                        ),
                        token.end_time.as_ref().unwrap(),
                    ));
                    continue;
                }

                if !timelines_by_name.contains_key(token.timeline_name)
                    || !problem.timelines[timelines_by_name[token.timeline_name]]
                        .values
                        .iter()
                        .any(|s| s.name == token.value)
                {
                    panic!("Shouldn't have added this state.");
                }

                // let timeline_name = problem.timelines[token.timeline_idx].name.as_str();
                // println!("Looking up {}.{}", token.timeline_name, token.value);

                let timeline_idx = timelines_by_name[token.timeline_name];
                if let Some(state) = problem.timelines[timeline_idx]
                    .values
                    .iter()
                    .find(|s| s.name == token.value)
                {
                    resource_constraints.entry(token_idx).or_default().capacity = Some(state.capacity);
                    // println!("Token {} {}", token.timeline_name, token.value);

                    // If there are old links pointing to this value, we need to update them.
                    // println!("Adding links for {}.{}", token.timeline_name, token.value);
                    for (link_idx, link) in links.iter().enumerate() {
                        if link.linkspec.value == token.value {
                            expand_links_queue.push((false, link_idx));
                        }
                    }

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
                        // assert!(condition.len() == 1, "Disjunctive goals not supported");
                        // println!("  -cond {} {:?}", token_idx, condition);
                        links.push(Link {
                            token_idx,
                            linkspec: condition,
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
                let (need_new, link_idx) = if link_queue < links.len() {
                    let link_idx = link_queue;
                    link_queue += 1;
                    (true, link_idx)
                } else {
                    // println!("Expanding link from expand queue.");
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

                let mut alternatives = Vec::new();

                let mut all_target_tokens = Vec::new();
                let mut new_target_tokens = Vec::new();
                for obj in objects.iter() {
                    if let Some(token_ref_list) = tokens_by_name.get(&(obj, &link.linkspec.value)) {
                        for token in token_ref_list.iter() {
                            all_target_tokens.push(*token);

                            if *token >= link.token_queue {
                                new_target_tokens.push(*token);
                            }
                        }
                    }
                }

                let link = &links[link_idx];
                if need_new && new_target_tokens.is_empty() {
                    // Select a object at random
                    // TODO make a heuristic for this

                    #[allow(clippy::never_loop)] // TODO multiplicity check will allow this to loop
                    for i in 0..objects.len() {
                        // if let Some(obj_name) = objects {

                        // This is a "random" heuristic for which object to expand.
                        let selected_object = (tokens.len() + links.len() + i) % objects.len();

                        let obj_name = objects[selected_object];
                        let new_token_idx = tokens.len();
                        //     let token_active = ;

                        if !timelines_by_name.contains_key(obj_name)
                            || !problem.timelines[timelines_by_name[obj_name]]
                                .values
                                .iter()
                                .any(|s| s.name == link.linkspec.value)
                        {
                            // This value cannot be created.
                            // Not possible to create this state (it's probably only used as a fact.)
                            continue;
                        }

                        // TODO check that multiplicity is not maxed.
                        tokens.push(Token {
                            timeline_name: obj_name,
                            active: Bool::fresh_const(&ctx, "pre"),
                            fact: false,
                            value: &link.linkspec.value,
                            start_time: Some(Real::fresh_const(
                                &ctx,
                                &format!("t_{}_{}_start_{}", obj_name, link.linkspec.value, new_token_idx),
                            )),
                            end_time: Some(Real::fresh_const(
                                &ctx,
                                &format!("t_{}_{}_end_{}", obj_name, link.linkspec.value, new_token_idx),
                            )),
                        });

                        new_target_tokens.push(new_token_idx);
                        tokens_by_name
                            .entry((obj_name, &link.linkspec.value))
                            .or_default()
                            .push(new_token_idx);
                        // }
                        break;
                    }
                }

                if need_new && new_target_tokens.is_empty() {
                    // println!(
                    //     "linking value {}.{} {}.{}\n --{:?}",
                    //     tokens[token_idx].timeline_name,
                    //     tokens[token_idx].value,
                    //     token.timeline_name,
                    //     token.value,
                    //     link.linkspec
                    // );
                }

                if !new_target_tokens.is_empty() {
                    let token = &tokens[link.token_idx];

                    for token_idx in new_target_tokens.iter().copied() {
                        // println!(
                        //     "linking value {}.{} {}.{}\n --{:?}",
                        //     tokens[token_idx].timeline_name,
                        //     tokens[token_idx].value,
                        //     token.timeline_name,
                        //     token.value,
                        //     link.linkspec
                        // );

                        // Represents the usage of the causal link.
                        let choose_link = Bool::fresh_const(&ctx, "cl");

                        let temporal_rel = match link.linkspec.temporal_relationship {
                            TemporalRelationship::MetBy => vec![Real::_eq(
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
                            TemporalRelationship::Meets => todo!(),
                        };

                        if link.linkspec.amount > 0 {
                        // println!("Link has amount {:?}", link.linkspec);
                            // Add resource constraint for this token.
                        let rc = resource_constraints.entry(token_idx).or_default();
                        assert!(!rc.closed);
                        rc.users
                            .push((choose_link.clone(), link.token_idx, link.linkspec.amount));
                        }

                        // The choose_link boolean implies all the condntions.
                        for cond in temporal_rel.iter().chain(std::iter::once(&tokens[token_idx].active)) {
                            solver.assert(&Bool::implies(&choose_link, cond));
                        }

                        alternatives.push(choose_link);
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

                    let old_expansion_lit: Option<Bool> = links[link_idx].alternatives_extension.take();

                    if let Some(b) = old_expansion_lit.as_ref() {
                        assert!(expand_links_lits.remove(b).is_some());
                    }

                    if total_multiplicity >= 2 {
                        let expand_lit = Bool::fresh_const(&ctx, "exp");
                        assert!(expand_links_lits.insert(expand_lit.clone(), link_idx).is_none());
                        links[link_idx].alternatives_extension = Some(expand_lit.clone());
                        alternatives.push(expand_lit);
                    } else {
                        // println!(
                        //     "NO MORE ALTERNATIVES FOR {} {}",
                        //     problem.timelines[token.timeline_idx].name, token.value
                        // );
                    }

                    let need_alternatives =
                        old_expansion_lit.unwrap_or_else(|| tokens[links[link_idx].token_idx].active.clone());
                    alternatives.push(Bool::not(&need_alternatives));

                    let alternatives_refs = alternatives.iter().collect::<Vec<_>>();
                    // println!(
                    //     "TOKEN LINKS for {}.{}[{}] has {} alternatives",
                    //     problem.timelines[tokens[link.token_idx].timeline_idx].name,
                    //     tokens[link.token_idx].value,
                    //     link.token_idx,
                    //     alternatives.len()
                    // );
                    solver.assert(&Bool::or(&ctx, &alternatives_refs));
                }

                links[link_idx].token_queue = tokens.len();
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

                if rc.integrated != 0 {
                    println!("WARNING: resource constraint users has been extended.");
                }

                rc.integrated = rc.users.len();

                if !rc.closed {
                    // TODO: make an extension point in the pseudo-boolean constraint for adding more usages later.
                }

                println!(
                    "Adding resource constraint for {}.{} with size {} capacity {:?}",
                    tokens[*_token_idx].timeline_name,
                    tokens[*_token_idx].value,
                    rc.users.len(),
                    rc.capacity
                );

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

                    // println!(
                    //     "Adding resource constraint for {}.{} with size {}",
                    //     tokens[*_token_idx].timeline_name,
                    //     tokens[*_token_idx].value,
                    //     overlaps.len()
                    // );
                    solver.assert(&Bool::pb_le(&ctx, &overlaps_refs, rc.capacity.unwrap() as i32));
                }
            }
        }

        if need_more_links_than > 0 && need_more_links_than == links.len() {
            // TODO this is not complete if we don't expand ALL of the core below. (but we do expand all, for now.)
            println!("Didn't expand any links!");
            return Err(SolverError::NoSolution);
        }

        let expand_links_negated_lits = expand_links_lits
            .keys()
            .map(|l| (Bool::not(l), l.clone()))
            .collect::<HashMap<_, _>>();

        let assumptions = expand_links_negated_lits.keys().cloned().collect::<Vec<_>>();
        // println!("{}", solver);
        // );
        let bakes_str = bakes
            .iter()
            .copied()
            .map(|t| format!("{}.{}", tokens[t].timeline_name, tokens[t].value))
            .collect::<Vec<_>>();
        println!("Bakes {}: {:?}", bakes.len(), bakes_str);
        let result = solver.check_assumptions(&assumptions);
        match result {
            z3::SatResult::Unsat => {
                #[allow(unused_mut)]
                let mut core = solver.get_unsat_core();
                if core.is_empty() {
                    return Err(SolverError::NoSolution);
                }

                // let use_trim_core = false;
                // let use_minimize_core = false;

                // if use_trim_core {
                //     crate::cores::trim_core(&mut core, &solver);
                // }

                // if use_minimize_core {
                //     crate::cores::minimize_core(&mut core, &solver);
                // }

                core_sizes.push(core.len());
                println!("CORE SIZE #{}: {:?}", core_sizes.len(), core_sizes);
                for c in core {
                    let link_idx = expand_links_lits[&expand_links_negated_lits[&c]]; //.remove(&c).unwrap();
                    let link = &links[link_idx];
                    let token = &tokens[link.token_idx];
                    println!("  -expand {}.{} {:?}", token.timeline_name, token.value, link.linkspec);

                    // TODO heuristically decide which and how many to expand.s
                    expand_links_queue.push((true, link_idx));
                    need_more_links_than = links.len();
                }
            }

            z3::SatResult::Sat => {
                let model = solver.get_model().unwrap();

                let mut solution_tokens = Vec::new();
                for v in tokens.iter() {
                    if !model.eval(&v.active, true).unwrap().as_bool().unwrap() {
                        continue;
                    }

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


struct Link<'a, 'z3> {
    token_idx: usize,
    linkspec: &'a Condition,
    token_queue: usize,
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
