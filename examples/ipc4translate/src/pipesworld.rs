use crate::{problem::*, SexpUnwrap};
use std::collections::{HashMap, HashSet};

/// Convert instance files for Pipesworld notankage temporal deadlines
/// This i an alternative encoding focusing on batches.
pub fn convert_pipesworld_batchencoding_notankage_temporal_deadlines() {
    let dir = "inputs/pipesworld";
    let domainfile = "DOMAIN.PDDL";
    let domain = sexp::parse(&std::fs::read_to_string(&format!("{}/{}", dir, domainfile)).unwrap()).unwrap();

    let mut products = Vec::new();

    let stmts = domain.unwrap_list().iter().collect::<Vec<_>>();
    assert!(stmts[0].unwrap_atom().to_string() == "define");
    assert!(stmts[1].unwrap_list()[0].to_string() == "domain");
    let domain_name = stmts[1].unwrap_list()[1].to_string();
    println!("Domain name: {}", domain_name);
    for stmt in stmts[2..].iter() {
        let stmt = stmt.unwrap_list();
        match stmt[0].unwrap_atom().to_string().as_str() {
            ":requirements" => {}
            ":types" => {}
            ":predicates" => {}
            ":functions" => {}
            ":constants" => {
                let mut objs = stmt[1..].iter().collect::<Vec<_>>();
                while objs.len() >= 3 {
                    let mut names = Vec::new();
                    let mut name = objs.remove(0).to_string().to_lowercase();
                    while name != "-" {
                        names.push(name);
                        name = objs.remove(0).to_string().to_lowercase();
                    }
                    let objtype = objs.remove(0);
                    println!("obj types {}", objtype);
                    match objtype.to_string().to_lowercase().as_str() {
                        "product" => products.extend(names),
                        _ => panic!(),
                    }
                }
            }
            ":durative-action" => {
                // println!("ACTION");
                // let stmt = &stmt[1..];
                // for s in stmt.iter() {
                //     // println!(" {}",s);
                // }
            }
            _ => {
                println!("UNKNOWN domain statement {:?}", stmt);
            }
        }
    }

    for file in std::fs::read_dir(dir).unwrap().flatten() {
        if file.file_name().to_str().unwrap() == domainfile {
            continue;
        }

        let instance =
            sexp::parse(&std::fs::read_to_string(format!("{}/{}", dir, file.file_name().to_str().unwrap())).unwrap())
                .unwrap();

        let stmts = instance.unwrap_list().iter().collect::<Vec<_>>();
        assert!(stmts[0].unwrap_atom().to_string().to_lowercase() == "define");
        assert!(stmts[1].unwrap_list()[0].to_string().to_lowercase() == "problem");
        let problem_name = stmts[1].unwrap_list()[1].to_string();
        println!("Problem name: {}", problem_name);

        let mut batch_atoms = Vec::new();
        let mut areas = Vec::new();
        let mut pipes = Vec::new();

        let mut deliverable = Vec::new();
        let mut normal = Vec::new();
        let mut may_interface = Vec::new();
        let mut connect = Vec::new();
        let mut is_product = Vec::new();
        let mut on = Vec::new();
        let mut first = Vec::new();
        let mut last = Vec::new();
        let mut follow = Vec::new();
        let mut unitary = Vec::new();
        let mut not_unitary = Vec::new();
        let mut speed = Vec::new();
        let mut on_goal = Vec::new();
        let mut not_deliverable = Vec::new();

        for stmt in stmts[2..].iter() {
            let stmt = stmt.unwrap_list();
            match stmt[0].unwrap_atom().to_string().to_lowercase().as_str() {
                ":domain" => {
                    assert!(stmt[1].unwrap_atom().to_string().to_lowercase() == domain_name);
                }
                ":objects" => {
                    let mut objs = stmt[1..].iter().collect::<Vec<_>>();
                    while objs.len() >= 3 {
                        let mut names = Vec::new();
                        let mut name = objs.remove(0).to_string().to_lowercase();
                        while name != "-" {
                            names.push(name);
                            name = objs.remove(0).to_string().to_lowercase();
                        }
                        let objtype = objs.remove(0);
                        println!("obj types {}", objtype);
                        match objtype.to_string().to_lowercase().as_str() {
                            "batch-atom" => batch_atoms.extend(names),
                            "area" => areas.extend(names),
                            "pipe" => pipes.extend(names),
                            _ => panic!(),
                        }
                    }
                    assert!(objs.is_empty());
                }
                ":init" => {
                    for initstmt in &stmt[1..] {
                        let stmt = initstmt.unwrap_list();
                        match stmt[0].unwrap_atom().to_string().to_lowercase().as_str() {
                            "deliverable" => deliverable.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            "normal" => normal.push((stmt[1].unwrap_atom().to_string().to_lowercase(),)),
                            "may-interface" => may_interface.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "connect" => connect.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                                stmt[3].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "is-product" => is_product.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "on" => on.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "last" => last.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "first" => first.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "follow" => follow.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "unitary" => unitary.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            "not-unitary" => not_unitary.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            "=" => {
                                let lhs = stmt[1].unwrap_list();
                                let rhs = match stmt[2].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };

                                match lhs[0].unwrap_atom().to_string().to_lowercase().as_str() {
                                    "speed" => {
                                        println!("Speed {}", stmt[1].to_string());
                                        speed.push((lhs[1].unwrap_atom().to_string().to_lowercase(), rhs));
                                    }
                                    _ => panic!(),
                                };
                            }
                            "at" => {
                                println!(" at   {:?}", stmt);
                                let t = match stmt[1].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };
                                let expr = stmt[2].unwrap_list();

                                // let not = if expr[0].unwrap_atom().to_string().as_str().to_lowercase() == "not" {
                                //     expr = expr[1].unwrap_list();
                                //     true
                                // } else {
                                //     false
                                // };

                                match expr[0].unwrap_atom().to_string().as_str() {
                                    "not" => {
                                        let v = expr[1].unwrap_list();
                                        match v[0].unwrap_atom().to_string().to_lowercase().as_str() {
                                            "deliverable" => {
                                                not_deliverable
                                                    .push((v[1].unwrap_atom().to_string().to_lowercase(), t));
                                            }
                                            _ => panic!(),
                                        }
                                    }
                                    _ => panic!(),
                                };
                            }
                            x => {
                                println!("Unknown init {}", x);
                                panic!();
                            }
                        }
                    }
                }
                ":goal" => {
                    let goals = stmt[1].unwrap_list();
                    assert!(goals[0].unwrap_atom().to_string().as_str() == "and");
                    for goal in goals[1..].iter() {
                        let goal = goal.unwrap_list();
                        match goal[0].unwrap_atom().to_string().as_str() {
                            "on" => {
                                let a = goal[1].unwrap_atom().to_string().to_lowercase();
                                let b = goal[2].unwrap_atom().to_string().to_lowercase();
                                on_goal.push((a, b));
                            }
                            _ => panic!("unknown goal type"),
                        }
                    }
                }
                ":metric" => {
                    // Ignoring optimizatoin.
                }
                _ => {
                    panic!("UNKNOWN instance statment");
                }
            }
        }

        println!("{} pipes {} goals", pipes.len(), on_goal.len());

        // Problem interpretation
        //

        // all batches should be deliverable in the beginning
        assert!(deliverable.iter().collect::<HashSet<_>>() == batch_atoms.iter().collect::<HashSet<_>>());
        println!("not_deliverable {:?}", not_deliverable);
        // let mut deliverable = Vec::new();
        // let mut not_deliverable = Vec::new();

        // Recreate the contents of each pipe from the first/last/follow relations
        // let mut first = Vec::new();
        // let mut last = Vec::new();
        // let mut follow = Vec::new();

        let mut pipe_state: HashMap<&String, Vec<&String>> = HashMap::new();
        for (batch, pipe) in first.iter() {
            assert!(pipe_state.insert(pipe, vec![batch]).is_none());
        }

        for (batch, prev_batch) in follow.iter() {
            let pipe = *pipe_state
                .iter()
                .find_map(|(p, s)| (s.last() == Some(&prev_batch)).then(|| p))
                .unwrap();

            pipe_state.get_mut(pipe).unwrap().push(batch);
        }

        for (batch, pipe) in last.iter() {
            println!("{:?}    {}", pipe_state, pipe);
            assert!(pipe_state[pipe].last() == Some(&batch));
        }

        let may_interface_set = may_interface
            .iter()
            .map(|(a, b)| (a, b))
            .collect::<HashSet<(&String, &String)>>();
        let is_product_map = is_product
            .iter()
            .map(|(a, b)| (a, b))
            .collect::<HashMap<&String, &String>>();

        let mut timelines: HashMap<String, Vec<TokenType>> = HashMap::new();
        let mut statictokens = Vec::new();

        // Deliverable deadlines.
        for batch in deliverable.iter() {
            let deadline = not_deliverable
                .iter()
                .find_map(|(b, t)| (b == batch).then(|| t))
                .map(|d| (d * 1000. + 0.5) as usize);

            statictokens.push(Token {
                capacity: 0,
                const_time: TokenTime::Fact(Some(0), deadline),
                timeline_name: format!("deliverable_{}", batch),
                value: "Yes".to_string(),
                conditions: vec![],
            });
        }

        // pipe parts are resources
        for (pipe, parts) in pipe_state.iter() {
            for (part_idx, _content) in parts.iter().enumerate() {
                timelines.insert(
                    format!("{}_part{}", pipe, part_idx),
                    vec![TokenType {
                        capacity: 1,
                        conditions: vec![],
                        duration: (1, None),
                        name: "Available".to_string(),
                    }],
                );
            }
        }
        for batch in batch_atoms.iter() {
            let tl_name = batch.clone();

            // INITIAL_VALUE

            if let Some(area) = on.iter().find_map(|(b, a)| (b == batch).then(|| a)) {
                statictokens.push(Token {
                    capacity: 0,
                    const_time: TokenTime::Fact(Some(0), None),
                    timeline_name: tl_name.clone(),
                    value: area.clone(),
                    conditions: vec![],
                });
            } else {
                let mut found = false;
                for (pipe, state) in pipe_state.iter() {
                    for (part_idx, b) in state.iter().enumerate() {
                        if *b == batch {
                            found = true;
                            let loc = format!("{}_part{}", pipe, part_idx);
                            statictokens.push(Token {
                                capacity: 0,
                                const_time: TokenTime::Fact(Some(0), None),
                                timeline_name: tl_name.clone(),
                                value: loc.clone(),
                                conditions: vec![Condition {
                                    // Only one content of the pipe.
                                    amount: 1,
                                    object: ObjectSet::Object(loc.clone()),
                                    temporal_relationship: TemporalRelationship::Cover,
                                    value: "Available".to_string(),
                                }],
                            });
                        }
                    }
                }

                assert!(found, "Batch has no initial state.");
            }
            // for (batch, area) in on.iter() {
            //     statictokens.push(Token {
            //         capacity: 0,
            //         const_time: TokenTime::Fact(None, None),
            //         timeline_name: batch.clone(),
            //         value: area.clone(),
            //         conditions: vec![],
            //     });
            // }

            let mut values = Vec::new();

            for area in areas.iter() {
                values.push(TokenType {
                    capacity: 0,
                    conditions: vec![],
                    duration: (1, None),
                    name: area.clone(),
                });
            }

            for (pipe, parts) in pipe_state.iter() {
                for (part_idx, _content) in parts.iter().enumerate() {
                    let pipe_part = format!("{}_part{}", pipe, part_idx);
                    values.push(TokenType {
                        capacity: 0,
                        conditions: vec![Condition {
                            // Only one content of the pipe.
                            amount: 1,
                            object: ObjectSet::Object(pipe_part.clone()),
                            temporal_relationship: TemporalRelationship::Cover,
                            value: "Available".to_string(),
                        }],
                        duration: (1, None),
                        name: pipe_part.clone(),
                    })
                }
            }

            // TRANSITIONS

            // AREA->PIPE
            for area in areas.iter() {
                for (a1, _, pipe) in connect.iter() {
                    if area == a1 {
                        let pipe_part = format!("{}_part{}", pipe, 0);

                        let pipe_speed = speed.iter().find_map(|(p, s)| (p == pipe).then(|| s)).unwrap();
                        let dur = ((1.0 / pipe_speed) * 1000.0 + 0.5) as usize;

                        values.push(TokenType {
                            name: format!("{}->{}", area, pipe_part),
                            duration: (dur, Some(dur)),
                            conditions: vec![
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                    value: area.clone(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    temporal_relationship: TemporalRelationship::Meets,
                                    value: pipe_part.clone(),
                                },
                            ],
                            capacity: 0,
                        });
                    }
                }

                for (_, a2, pipe) in connect.iter() {
                    if area == a2 {
                        let pipe_part = format!("{}_part{}", pipe, pipe_state[pipe].len() - 1);
                        let pipe_speed = speed.iter().find_map(|(p, s)| (p == pipe).then(|| s)).unwrap();
                        let dur = ((1.0 / pipe_speed) * 1000.0 + 0.5) as usize;
                        values.push(TokenType {
                            name: format!("{}->{}", area, pipe_part),
                            duration: (dur, Some(dur)),
                            conditions: vec![
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                    value: area.clone(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    temporal_relationship: TemporalRelationship::Meets,
                                    value: pipe_part.clone(),
                                },
                            ],
                            capacity: 0,
                        });
                    }
                }
            }

            // PIPE->PIPE
            for (pipe, state) in pipe_state.iter() {
                for i in 0..(state.len() - 1) {
                    // Something needs to fill part I afterwards.

                    let prev_part = if i > 0 {
                        format!("{}_part{}", pipe, i - 1)
                    } else {
                        connect
                            .iter()
                            .find_map(|(a, _, p)| (p == *pipe).then(|| a))
                            .unwrap()
                            .clone()
                    };

                    let next_part = if i + 2 < pipe_state.len() {
                        format!("{}_part{}", pipe, i + 2)
                    } else {
                        connect
                            .iter()
                            .find_map(|(_, a, p)| (p == *pipe).then(|| a))
                            .unwrap()
                            .clone()
                    };

                    let part1 = format!("{}_part{}", pipe, i);
                    let part2 = format!("{}_part{}", pipe, i + 1);
                    let pipe_speed = speed.iter().find_map(|(p, s)| (p == *pipe).then(|| s)).unwrap();
                    let dur = ((1.0 / pipe_speed) * 1000.0 + 0.5) as usize;

                    let compatible_batches: Vec<String> = batch_atoms
                        .iter()
                        .filter(|b| {
                            *b != batch && may_interface_set.contains(&(is_product_map[*b], is_product_map[batch]))
                        })
                        .cloned()
                        .collect();

                    values.push(TokenType {
                        name: format!("{}->{}", part1, part2),
                        duration: (dur, Some(dur)),
                        conditions: vec![
                            Condition {
                                amount: 0,
                                object: ObjectSet::Object(tl_name.clone()),
                                temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                value: part1.clone(),
                            },
                            Condition {
                                amount: 0,
                                object: ObjectSet::Object(tl_name.clone()),
                                temporal_relationship: TemporalRelationship::Meets,
                                value: part2.clone(),
                            },
                            // Condition {
                            //     amount: 0,
                            //     object: ObjectSet::Set(compatible_batches.clone()),
                            //     temporal_relationship: TemporalRelationship::Meets,
                            //     value: part1.clone(),
                            // },
                            // ALTERNATIVE FORMULATION:
                            //  find the previous P and set Condition Group(Batches) val:P->part1  rel:Equal.
                            Condition {
                                amount: 0,
                                object: ObjectSet::Set(compatible_batches.clone()),
                                temporal_relationship: TemporalRelationship::Cover,
                                value: format!("{}->{}", prev_part, part1.clone()),
                            },
                        ],
                        capacity: 0,
                    });

                    values.push(TokenType {
                        name: format!("{}->{}", part2, part1),
                        duration: (dur, Some(dur)),
                        conditions: vec![
                            Condition {
                                amount: 0,
                                object: ObjectSet::Object(tl_name.clone()),
                                temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                value: part2.clone(),
                            },
                            Condition {
                                amount: 0,
                                object: ObjectSet::Object(tl_name.clone()),
                                temporal_relationship: TemporalRelationship::Meets,
                                value: part1.clone(),
                            },
                            // Condition {
                            //     amount: 0,
                            //     object: ObjectSet::Set(compatible_batches.clone()),
                            //     temporal_relationship: TemporalRelationship::Meets,
                            //     value: part2.clone(),
                            // },
                            Condition {
                                amount: 0,
                                object: ObjectSet::Set(compatible_batches.clone()),
                                temporal_relationship: TemporalRelationship::Cover,
                                value: format!("{}->{}", next_part, part2.clone()),
                            },
                        ],
                        capacity: 0,
                    });
                }
            }

            // PIPE->AREA
            for area in areas.iter() {
                for (a1, a2, pipe) in connect.iter() {
                    if area == a1 {
                        let pipe_part = format!("{}_part{}", pipe, 0);

                        let next_part = if 1 < pipe_state[pipe].len() {
                            format!("{}_part{}", pipe, 1)
                        } else {
                            a2.clone()
                        };

                        let pipe_speed = speed.iter().find_map(|(p, s)| (p == pipe).then(|| s)).unwrap();
                        let dur = ((1.0 / pipe_speed) * 1000.0 + 0.5) as usize;

                        let compatible_batches: Vec<String> = batch_atoms
                            .iter()
                            .filter(|b| {
                                *b != batch && may_interface_set.contains(&(is_product_map[*b], is_product_map[batch]))
                            })
                            .cloned()
                            .collect();

                        values.push(TokenType {
                            name: format!("{}->{}", pipe_part, area),
                            duration: (dur, Some(dur)),
                            conditions: vec![
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(format!("deliverable_{}", batch)),
                                    temporal_relationship: TemporalRelationship::Cover,
                                    value: "Yes".to_string(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                    value: pipe_part.clone(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    temporal_relationship: TemporalRelationship::Meets,
                                    value: area.clone(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Set(compatible_batches.clone()),
                                    temporal_relationship: TemporalRelationship::Cover,
                                    value: format!("{}->{}", next_part, pipe_part),
                                },
                            ],
                            capacity: 0,
                        });
                    }
                }

                for (a1, a2, pipe) in connect.iter() {
                    if area == a2 {
                        let part_idx = pipe_state[pipe].len() - 1;
                        let pipe_part = format!("{}_part{}", pipe, part_idx);

                        let next_part = if part_idx > 0 {
                            format!("{}_part{}", pipe, part_idx - 1)
                        } else {
                            a1.clone()
                        };

                        let pipe_speed = speed.iter().find_map(|(p, s)| (p == pipe).then(|| s)).unwrap();
                        let dur = ((1.0 / pipe_speed) * 1000.0 + 0.5) as usize;

                        let compatible_batches: Vec<String> = batch_atoms
                            .iter()
                            .filter(|b| {
                                *b != batch && may_interface_set.contains(&(is_product_map[*b], is_product_map[batch]))
                            })
                            .cloned()
                            .collect();

                        values.push(TokenType {
                            name: format!("{}->{}", pipe_part, area),
                            duration: (dur, Some(dur)),
                            conditions: vec![
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(format!("deliverable_{}", batch)),
                                    temporal_relationship: TemporalRelationship::Cover,
                                    value: "Yes".to_string(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                    value: pipe_part.clone(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    temporal_relationship: TemporalRelationship::Meets,
                                    value: area.clone(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Set(compatible_batches.clone()),
                                    temporal_relationship: TemporalRelationship::Cover,
                                    value: format!("{}->{}", next_part, pipe_part.clone()),
                                },
                            ],
                            capacity: 0,
                        });
                    }
                }
            }

            timelines.insert(tl_name, values);
        }

        for (batch, area) in on_goal.iter() {
            statictokens.push(Token {
                capacity: 0,
                const_time: TokenTime::Goal,
                timeline_name: batch.clone(),
                value: area.clone(),
                conditions: vec![],
            });
        }

        let problem = Problem {
            groups: vec![Group {
                name: "Batches".to_string(),
                members: batch_atoms.clone(),
            }],
            timelines: timelines
                .into_iter()
                .map(|(n, v)| Timeline { name: n, values: v })
                .collect(),
            tokens: statictokens,
        };
        let json = serde_json::to_string(&problem).unwrap();
        std::fs::write(
            &format!("pipesworldalt_{}.json", file.file_name().to_str().unwrap()),
            &json,
        )
        .unwrap();
    }
}

pub fn convert_pipesworld_notankage_temporal_deadlines() {
    let dir = "inputs/pipesworld";
    let domainfile = "DOMAIN.PDDL";
    let domain = sexp::parse(&std::fs::read_to_string(&format!("{}/{}", dir, domainfile)).unwrap()).unwrap();

    let mut products = Vec::new();

    let stmts = domain.unwrap_list().iter().collect::<Vec<_>>();
    assert!(stmts[0].unwrap_atom().to_string() == "define");
    assert!(stmts[1].unwrap_list()[0].to_string() == "domain");
    let domain_name = stmts[1].unwrap_list()[1].to_string();
    println!("Domain name: {}", domain_name);
    for stmt in stmts[2..].iter() {
        let stmt = stmt.unwrap_list();
        match stmt[0].unwrap_atom().to_string().as_str() {
            ":requirements" => {}
            ":types" => {}
            ":predicates" => {}
            ":functions" => {}
            ":constants" => {
                let mut objs = stmt[1..].iter().collect::<Vec<_>>();
                while objs.len() >= 3 {
                    let mut names = Vec::new();
                    let mut name = objs.remove(0).to_string().to_lowercase();
                    while name != "-" {
                        names.push(name);
                        name = objs.remove(0).to_string().to_lowercase();
                    }
                    let objtype = objs.remove(0);
                    println!("obj types {}", objtype);
                    match objtype.to_string().to_lowercase().as_str() {
                        "product" => products.extend(names),
                        _ => panic!(),
                    }
                }
            }
            ":durative-action" => {
                // println!("ACTION");
                // let stmt = &stmt[1..];
                // for s in stmt.iter() {
                //     // println!(" {}",s);
                // }
            }
            _ => {
                println!("UNKNOWN domain statement {:?}", stmt);
            }
        }
    }

    for file in std::fs::read_dir(dir).unwrap().flatten() {
        if file.file_name().to_str().unwrap() == domainfile {
            continue;
        }

        let instance =
            sexp::parse(&std::fs::read_to_string(format!("{}/{}", dir, file.file_name().to_str().unwrap())).unwrap())
                .unwrap();

        let stmts = instance.unwrap_list().iter().collect::<Vec<_>>();
        assert!(stmts[0].unwrap_atom().to_string().to_lowercase() == "define");
        assert!(stmts[1].unwrap_list()[0].to_string().to_lowercase() == "problem");
        let problem_name = stmts[1].unwrap_list()[1].to_string();
        println!("Problem name: {}", problem_name);

        let mut batch_atoms = Vec::new();
        let mut areas = Vec::new();
        let mut pipes = Vec::new();

        let mut deliverable = Vec::new();
        let mut normal = Vec::new();
        let mut may_interface = Vec::new();
        let mut connect = Vec::new();
        let mut is_product = Vec::new();
        let mut on = Vec::new();
        let mut first = Vec::new();
        let mut last = Vec::new();
        let mut follow = Vec::new();
        let mut unitary = Vec::new();
        let mut not_unitary = Vec::new();
        let mut speed = Vec::new();
        let mut on_goal = Vec::new();
        let mut not_deliverable = Vec::new();

        for stmt in stmts[2..].iter() {
            let stmt = stmt.unwrap_list();
            match stmt[0].unwrap_atom().to_string().to_lowercase().as_str() {
                ":domain" => {
                    assert!(stmt[1].unwrap_atom().to_string().to_lowercase() == domain_name);
                }
                ":objects" => {
                    let mut objs = stmt[1..].iter().collect::<Vec<_>>();
                    while objs.len() >= 3 {
                        let mut names = Vec::new();
                        let mut name = objs.remove(0).to_string().to_lowercase();
                        while name != "-" {
                            names.push(name);
                            name = objs.remove(0).to_string().to_lowercase();
                        }
                        let objtype = objs.remove(0);
                        println!("obj types {}", objtype);
                        match objtype.to_string().to_lowercase().as_str() {
                            "batch-atom" => batch_atoms.extend(names),
                            "area" => areas.extend(names),
                            "pipe" => pipes.extend(names),
                            _ => panic!(),
                        }
                    }
                    assert!(objs.is_empty());
                }
                ":init" => {
                    for initstmt in &stmt[1..] {
                        let stmt = initstmt.unwrap_list();
                        match stmt[0].unwrap_atom().to_string().to_lowercase().as_str() {
                            "deliverable" => deliverable.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            "normal" => normal.push((stmt[1].unwrap_atom().to_string().to_lowercase(),)),
                            "may-interface" => may_interface.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "connect" => connect.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                                stmt[3].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "is-product" => is_product.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "on" => on.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "last" => last.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "first" => first.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "follow" => follow.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "unitary" => unitary.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            "not-unitary" => not_unitary.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            "=" => {
                                let lhs = stmt[1].unwrap_list();
                                let rhs = match stmt[2].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };

                                match lhs[0].unwrap_atom().to_string().to_lowercase().as_str() {
                                    "speed" => {
                                        println!("Speed {}", stmt[1].to_string());
                                        speed.push((lhs[1].unwrap_atom().to_string().to_lowercase(), rhs));
                                    }
                                    _ => panic!(),
                                };
                            }
                            "at" => {
                                println!(" at   {:?}", stmt);
                                let t = match stmt[1].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };
                                let expr = stmt[2].unwrap_list();

                                // let not = if expr[0].unwrap_atom().to_string().as_str().to_lowercase() == "not" {
                                //     expr = expr[1].unwrap_list();
                                //     true
                                // } else {
                                //     false
                                // };

                                match expr[0].unwrap_atom().to_string().as_str() {
                                    "not" => {
                                        let v = expr[1].unwrap_list();
                                        match v[0].unwrap_atom().to_string().to_lowercase().as_str() {
                                            "deliverable" => {
                                                not_deliverable
                                                    .push((v[1].unwrap_atom().to_string().to_lowercase(), t));
                                            }
                                            _ => panic!(),
                                        }
                                    }
                                    _ => panic!(),
                                };
                            }
                            x => {
                                println!("Unknown init {}", x);
                                panic!();
                            }
                        }
                    }
                }
                ":goal" => {
                    let goals = stmt[1].unwrap_list();
                    assert!(goals[0].unwrap_atom().to_string().as_str() == "and");
                    for goal in goals[1..].iter() {
                        let goal = goal.unwrap_list();
                        match goal[0].unwrap_atom().to_string().as_str() {
                            "on" => {
                                let a = goal[1].unwrap_atom().to_string().to_lowercase();
                                let b = goal[2].unwrap_atom().to_string().to_lowercase();
                                on_goal.push((a, b));
                            }
                            _ => panic!("unknown goal type"),
                        }
                    }
                }
                ":metric" => {
                    // Ignoring optimizatoin.
                }
                _ => {
                    panic!("UNKNOWN instance statment");
                }
            }
        }

        println!("{} pipes {} goals", pipes.len(), on_goal.len());

        // Problem interpretation
        //

        let mut timelines: HashMap<String, Vec<TokenType>> = HashMap::new();
        let mut statictokens = Vec::new();

        // all batches should be deliverable in the beginning
        assert!(deliverable.iter().collect::<HashSet<_>>() == batch_atoms.iter().collect::<HashSet<_>>());
        println!("not_deliverable {:?}", not_deliverable);
        // let mut deliverable = Vec::new();
        // let mut not_deliverable = Vec::new();

        for batch in deliverable.iter() {
            let deadline = not_deliverable
                .iter()
                .find_map(|(b, t)| (b == batch).then(|| t))
                .map(|d| (d * 1000. + 0.5) as usize);

            statictokens.push(Token {
                capacity: 0,
                const_time: TokenTime::Fact(Some(0), deadline),
                timeline_name: format!("deliverable_{}", batch),
                value: "Yes".to_string(),
                conditions: vec![],
            });
        }

        // Recreate the contents of each pipe from the first/last/follow relations
        // let mut first = Vec::new();
        // let mut last = Vec::new();
        // let mut follow = Vec::new();

        let mut pipe_state: HashMap<&String, Vec<&String>> = HashMap::new();
        for (batch, pipe) in first.iter() {
            assert!(pipe_state.insert(pipe, vec![batch]).is_none());
        }

        for (batch, prev_batch) in follow.iter() {
            let pipe = *pipe_state
                .iter()
                .find_map(|(p, s)| (s.last() == Some(&prev_batch)).then(|| p))
                .unwrap();

            pipe_state.get_mut(pipe).unwrap().push(batch);
        }

        for (batch, pipe) in last.iter() {
            println!("{:?}    {}", pipe_state, pipe);
            assert!(pipe_state[pipe].last() == Some(&batch));
        }

        for batch in batch_atoms.iter() {
            let tl_name = batch.clone();
            let mut values = Vec::new();

            for area in areas.iter() {
                values.push(TokenType {
                    duration: (1, None),
                    name: area.clone(),
                    capacity: 0,
                    conditions: vec![],
                })
            }

            for pipe in pipes.iter() {
                values.push(TokenType {
                    duration: (1, None),
                    name: pipe.clone(),
                    capacity: 0,
                    conditions: vec![],
                })
            }

            for pipe in pipes.iter() {
                println!("{:?} piep {:?} connect", pipes, connect);
                let (a, b) = connect
                    .iter()
                    .find_map(|(a, b, p)| (p == pipe).then(|| (a, b)))
                    .unwrap();

                let pipe_a = format!("{}_part{}", pipe, 0);
                let pipe_b = format!("{}_part{}", pipe, pipe_state[pipe].len() - 1);

                values.push(TokenType {
                    duration: (1, None),
                    name: format!("{}-{}", a, pipe),
                    capacity: 0,
                    conditions: vec![
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            value: a.clone(),
                            temporal_relationship: TemporalRelationship::MetBy,
                        },
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            value: pipe.clone(),
                            temporal_relationship: TemporalRelationship::Meets,
                        },
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(pipe_a.clone()),
                            value: batch.clone(),
                            temporal_relationship: TemporalRelationship::Meets,
                        },
                    ],
                });
                values.push(TokenType {
                    duration: (1, None),
                    name: format!("{}-{}", pipe, a),
                    capacity: 0,
                    conditions: vec![
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            value: pipe.clone(),
                            temporal_relationship: TemporalRelationship::MetBy,
                        },
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            value: a.clone(),
                            temporal_relationship: TemporalRelationship::Meets,
                        },
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(pipe_a.clone()),
                            value: batch.clone(),
                            temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                        },
                    ],
                });
                values.push(TokenType {
                    duration: (1, None),
                    name: format!("{}-{}", pipe, b),
                    capacity: 0,
                    conditions: vec![
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            value: pipe.clone(),
                            temporal_relationship: TemporalRelationship::MetBy,
                        },
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            value: b.clone(),
                            temporal_relationship: TemporalRelationship::Meets,
                        },
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(pipe_b.clone()),
                            value: batch.clone(),
                            temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                        },
                    ],
                });
                values.push(TokenType {
                    duration: (1, None),
                    name: format!("{}-{}", b, pipe),
                    capacity: 0,
                    conditions: vec![
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            value: b.clone(),
                            temporal_relationship: TemporalRelationship::MetBy,
                        },
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            value: pipe.clone(),
                            temporal_relationship: TemporalRelationship::Meets,
                        },
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(pipe_b.clone()),
                            value: batch.clone(),
                            temporal_relationship: TemporalRelationship::Meets,
                        },
                    ],
                });
            }

            timelines.insert(tl_name, values);
        }

        // let mut batch_atoms = Vec::new();
        // let mut areas = Vec::new();
        // let mut pipes = Vec::new();
        // let mut on = Vec::new();

        for (batch, area) in on.iter() {
            statictokens.push(Token {
                capacity: 0,
                const_time: TokenTime::Fact(None, None),
                timeline_name: batch.clone(),
                value: area.clone(),
                conditions: vec![],
            });
        }

        // unitary-nonunitary matches pipe_state lengths
        // let mut unitary = Vec::new();
        // let mut not_unitary = Vec::new();

        assert!(pipe_state.iter().all(|(p, s)| if s.len() == 1 {
            unitary.iter().any(|u| *p == u)
        } else {
            not_unitary.iter().any(|u| *p == u)
        }));

        println!("pipe states {:?}", pipe_state);

        let may_interface_set = may_interface
            .iter()
            .map(|(a, b)| (a, b))
            .collect::<HashSet<(&String, &String)>>();
        let is_product_map = is_product
            .iter()
            .map(|(a, b)| (a, b))
            .collect::<HashMap<&String, &String>>();

        let mut prev_location = HashMap::new();
        let mut next_location = HashMap::new();
        for (l1, l2, pipe) in connect.iter() {
            let pipe_speed = speed.iter().find_map(|(p, s)| (p == pipe).then(|| s)).unwrap();

            let mut locations = vec![Err(l1.clone())];
            for part in 0..pipe_state[pipe].len() {
                let name = format!("{}_part{}", pipe, part);
                locations.push(Ok(name.clone()));
            }
            locations.push(Err(l2.clone()));

            for (l1, l2) in locations.iter().zip(locations.iter().skip(1)) {
                if let Ok(a) = l1 {
                    next_location.insert(a.clone(), l2.clone());
                }
                if let Ok(b) = l2 {
                    prev_location.insert(b.clone(), l1.clone());
                }
            }

            for part in 0..pipe_state[pipe].len() {
                let name = format!("{}_part{}", pipe, part);
                let prev = &prev_location[&name];
                let next = &next_location[&name];

                statictokens.push(Token {
                    capacity: 0,
                    const_time: TokenTime::Fact(Some(0), None),
                    timeline_name: name.clone(),
                    value: pipe_state[pipe][part].clone(),
                    conditions: vec![],
                });
                statictokens.push(Token {
                    capacity: 0,
                    const_time: TokenTime::Fact(Some(0), None),
                    timeline_name: pipe_state[pipe][part].clone(),
                    value: pipe.clone(),
                    conditions: vec![],
                });

                let mut values = Vec::new();

                for batch in batch_atoms.iter() {
                    values.push(TokenType {
                        capacity: 0,
                        conditions: vec![],
                        duration: (1, None),
                        name: batch.clone(),
                    });
                }

                for b1 in batch_atoms.iter() {
                    for b2 in batch_atoms.iter() {
                        if b1 == b2 {
                            continue;
                        }
                        let product1 = is_product_map[b1];
                        let product2 = is_product_map[b2];
                        if !may_interface_set.contains(&(product1, product2)) {
                            continue;
                        }

                        let dur = ((1.0 / pipe_speed) * 1000.0 + 0.5) as usize;

                        let c_prevstate = Condition {
                            amount: 0,
                            object: ObjectSet::Object(name.clone()),
                            temporal_relationship: TemporalRelationship::MetBy,
                            value: b1.clone(),
                        };

                        let c_nextstate = Condition {
                            amount: 0,
                            object: ObjectSet::Object(name.clone()),
                            temporal_relationship: TemporalRelationship::Meets,
                            value: b2.clone(),
                        };

                        let c_prevpart_f = match prev {
                            Ok(part) => Condition {
                                amount: 0,
                                object: ObjectSet::Object(part.clone()),
                                temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                value: b2.clone(),
                            },
                            Err(area) => Condition {
                                amount: 0,
                                object: ObjectSet::Object(b2.clone()),
                                temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                value: area.clone(),
                            },
                        };

                        let c_nextpart_f = match next {
                            Ok(part) => vec![Condition {
                                amount: 0,
                                object: ObjectSet::Object(part.clone()),
                                temporal_relationship: TemporalRelationship::Meets,
                                value: b1.clone(),
                            }],
                            Err(area) => vec![
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(b1.clone()),
                                    temporal_relationship: TemporalRelationship::Meets,
                                    value: area.clone(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(format!("deliverable_{}", b1)),
                                    temporal_relationship: TemporalRelationship::Cover,
                                    value: "Yes".to_string(),
                                },
                            ],
                        };

                        let c_prevpart_b = match next {
                            Ok(part) => Condition {
                                amount: 0,
                                object: ObjectSet::Object(part.clone()),
                                temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                value: b2.clone(),
                            },
                            Err(area) => Condition {
                                amount: 0,
                                object: ObjectSet::Object(b2.clone()),
                                temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                value: area.clone(),
                            },
                        };

                        let c_nextpart_b = match prev {
                            Ok(part) => vec![Condition {
                                amount: 0,
                                object: ObjectSet::Object(part.clone()),
                                temporal_relationship: TemporalRelationship::Meets,
                                value: b1.clone(),
                            }],
                            Err(area) => vec![
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(b1.clone()),
                                    temporal_relationship: TemporalRelationship::Meets,
                                    value: area.clone(),
                                },
                                Condition {
                                    amount: 0,
                                    object: ObjectSet::Object(format!("deliverable_{}", b1)),
                                    temporal_relationship: TemporalRelationship::Cover,
                                    value: "Yes".to_string(),
                                },
                            ],
                        };

                        // EXCHANGE FORWARD
                        let mut forward_conditions = vec![c_prevstate.clone(), c_nextstate.clone(), c_prevpart_f];
                        forward_conditions.extend(c_nextpart_f);
                        values.push(TokenType {
                            name: format!("{}--{}--forward", b1, b2),
                            capacity: 0,
                            conditions: forward_conditions,
                            duration: (dur, Some(dur)),
                        });

                        // EXCHANGE BACKWARD
                        let mut backward_conditions = vec![c_prevstate, c_nextstate, c_prevpart_b];
                        backward_conditions.extend(c_nextpart_b);
                        values.push(TokenType {
                            name: format!("{}--{}--backward", b1, b2),
                            capacity: 0,
                            conditions: backward_conditions,
                            duration: (dur, Some(dur)),
                        });
                    }
                }

                timelines.insert(name, values);
            }
        }

        // // Dynamic pipe states
        // for pipe in pipes.iter() {
        //     let tl_name = pipe.clone();
        //     let mut values = Vec::new();

        //     statictokens.push(Token {
        //         capacity: 0,
        //         const_time: TokenTime::Fact(Some(0), None),
        //         timeline_name: tl_name.clone(),
        //         value: pipe_state[pipe]
        //             .iter()
        //             .copied()
        //             .cloned()
        //             .collect::<Vec<String>>()
        //             .join(";"),
        //     });

        //     let mut n = 0;
        //     'permutations: for bs in batch_atoms.iter().permutations(pipe_state[pipe].len()) {
        //         // Check if the itnerfaces are ok

        //         for (b1, b2) in bs.iter().zip(bs.iter().skip(1)) {
        //             let product1 = is_product_map[b1];
        //             let product2 = is_product_map[b2];
        //             if !may_interface_set.contains(&(product1, product2)) {
        //                 continue 'permutations;
        //             }
        //         }

        //         let value = bs.iter().copied().cloned().collect::<Vec<String>>().join(";");

        //         values.push(TokenType {
        //             name: value,
        //             capacity: 0,
        //             conditions: vec![],
        //             duration: (1, None),
        //         });

        //         n += 1;
        //     }

        //     println!("piep {} ahs {} states", pipe, n);

        //     assert!(timelines.insert(tl_name, values).is_none());
        // }

        // let mut normal = Vec::new();
        // let mut may_interface = Vec::new();
        // let mut connect = Vec::new();
        // let mut is_product = Vec::new();
        // let mut speed = Vec::new();
        // let mut on_goal = Vec::new();

        for (batch, area) in on_goal.iter() {
            statictokens.push(Token {
                capacity: 0,
                const_time: TokenTime::Goal,
                timeline_name: batch.clone(),
                value: area.clone(),
                conditions: vec![],
            });
        }

        let problem = Problem {
            groups: Vec::new(),
            timelines: timelines
                .into_iter()
                .map(|(n, v)| Timeline { name: n, values: v })
                .collect(),
            tokens: statictokens,
        };
        let json = serde_json::to_string(&problem).unwrap();
        std::fs::write(
            &format!("pipesworld_{}.json", file.file_name().to_str().unwrap()),
            &json,
        )
        .unwrap();
    }
}
