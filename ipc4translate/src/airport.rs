use itertools::Itertools;

use crate::{problem::*, SexpUnwrap};
use std::collections::{HashMap, HashSet};

/// Convert instance files for Pipesworld notankage temporal deadlines
pub fn convert_airport() {
    let dir = "inputs/airport";
    let domainfile = "DOMAIN.PDDL";
    let domain = sexp::parse(&std::fs::read_to_string(&format!("{}/{}", dir, domainfile)).unwrap()).unwrap();

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
                        _ => panic!(),
                    }
                }
            }
            ":durative-action" => {
                // println!("ACTION");
                let stmt = &stmt[1..];
                for s in stmt.iter() {
                    // println!(" {}",s);
                }
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

        let mut airplanes = Vec::new();
        let mut airplane_types = Vec::new();
        let mut directions = Vec::new();
        let mut segments = Vec::new();

        let mut at_segments = Vec::new();
        let mut blocked = Vec::new();
        let mut blocked_intervals = Vec::new();
        let mut can_move = Vec::new();
        let mut can_pushback = Vec::new();
        let mut facing = Vec::new();
        let mut has_type = Vec::new();
        let mut is_blocked = Vec::new();
        let mut is_moving = Vec::new();
        let mut is_pushing = Vec::new();
        let mut is_start_runway = Vec::new();
        let mut move_back_dir = Vec::new();
        let mut move_dir = Vec::new();
        let mut occupied = Vec::new();
        let mut length = Vec::new();
        let mut engines = Vec::new();

        let mut goal_parked = Vec::new();
        let mut goal_airborne = Vec::new();

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
                        // println!("obj types {}", objtype);
                        match objtype.to_string().to_lowercase().as_str() {
                            "airplane" => airplanes.extend(names),
                            "airplanetype" => airplane_types.extend(names),
                            "direction" => directions.extend(names),
                            "segment" => segments.extend(names),
                            _ => panic!(),
                        }
                    }
                    assert!(objs.is_empty());
                }
                ":init" => {
                    for initstmt in &stmt[1..] {
                        let stmt = initstmt.unwrap_list();
                        match stmt[0].unwrap_atom().to_string().to_lowercase().as_str() {
                            // "deliverable" => deliverable.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            // "normal" => normal.push((stmt[1].unwrap_atom().to_string().to_lowercase(),)),
                            "at-segment" => at_segments.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "blocked" => blocked.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "can-move" => can_move.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                                stmt[3].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "can-pushback" => can_pushback.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                                stmt[3].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "facing" => facing.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "has-type" => has_type.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "is-blocked" => is_blocked.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                                stmt[3].unwrap_atom().to_string().to_lowercase(),
                                stmt[4].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "is-moving" => is_moving.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            "is-pushing" => is_pushing.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            "is-start-runway" => is_start_runway.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "move-dir" => move_dir.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                                stmt[3].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "move-back-dir" => move_back_dir.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                                stmt[3].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "occupied" => occupied.push(stmt[1].unwrap_atom().to_string().to_lowercase()),

                            "=" => {
                                let lhs = stmt[1].unwrap_list();
                                let rhs = match stmt[2].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };

                                match lhs[0].unwrap_atom().to_string().to_lowercase().as_str() {
                                    "length" => {
                                        // println!("Speed {}", stmt[1].to_string());
                                        length.push((lhs[1].unwrap_atom().to_string().to_lowercase(), rhs));
                                    }
                                    "engines" => {
                                        // println!("Speed {}", stmt[1].to_string());
                                        engines.push((lhs[1].unwrap_atom().to_string().to_lowercase(), rhs));
                                    }
                                    _ => panic!(),
                                };
                            }
                            "at" => {
                                // println!(" at   {:?}", stmt);
                                let t = match stmt[1].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };
                                let mut expr = stmt[2].unwrap_list();

                                let (v, not) = if expr[0].unwrap_atom().to_string().to_lowercase().as_str() == "not" {
                                    (expr[1].unwrap_list(), true)
                                } else {
                                    (expr, false)
                                };

                                match v[0].unwrap_atom().to_string().to_lowercase().as_str() {
                                    "blocked" => {
                                        let x1 = v[1].unwrap_atom().to_string().to_lowercase();
                                        let x2 = v[2].unwrap_atom().to_string().to_lowercase();
                                        blocked_intervals.push((not, x1, x2, t));
                                    }
                                    _ => panic!(),
                                }
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
                            "is-parked" => {
                                let a = goal[1].unwrap_atom().to_string().to_lowercase();
                                let b = goal[2].unwrap_atom().to_string().to_lowercase();
                                goal_parked.push((a, b));
                            }
                            "airborne" => {
                                let a = goal[1].unwrap_atom().to_string().to_lowercase();
                                let b = goal[2].unwrap_atom().to_string().to_lowercase();
                                goal_airborne.push((a, b));
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

        println!(
            "{} planes {} segments {} directions",
            airplanes.len(),
            segments.len(),
            directions.len()
        );

        // ACTION ::: MOVE  -- AIRPLANE, DIR1, DIR2, SEG1, SEG2
        //   condition: airplane is moving
        //              can-move, move-dir
        //              airplane is at S1
        //              S2 is free (not blocked)
        //   blocked(?s,AIRPLANE.type,SEG2,DIR2) && ?s != SEG1   =>  !occupied(?s)
        //

        // ACTION ::: PUSHBACK -- AIRPLANE, DIR1, DIR2, SEG1, SEG2
        //

        // ACTION ::: STARTUP
        //  airplane goes from pushing to moving.¨

        // ACTION ::: PARK
        //  airplane goes from moving to parked.

        let mut timelines: HashMap<String, Vec<TokenType>> = HashMap::new();
        let mut statictokens = Vec::new();

        for airplane in airplanes.iter() {
            let tl_name = format!("{}_mode", airplane);

            if is_pushing.iter().any(|a| a == airplane) {
                statictokens.push(Token {
                    capacity: 0,
                    const_time: TokenTime::Fact(None, None),
                    timeline_name: tl_name.clone(),
                    value: "pushing".to_string(),
                });
            } else if is_moving.iter().any(|a| a == airplane) {
                statictokens.push(Token {
                    capacity: 0,
                    const_time: TokenTime::Fact(None, None),
                    timeline_name: tl_name.clone(),
                    value: "moving".to_string(),
                });
            } else {
                // Dummy airplane?
                println!("dummy {}", airplane);
                continue;
            }

            let startup_time = engines.iter().find_map(|(a, t)| (a == airplane).then(|| *t)).unwrap();
            let startup_time = (1000.0 * startup_time + 0.5) as usize;

            let mut values = vec![
                TokenType {
                    capacity: 0,
                    conditions: vec![],
                    duration: (startup_time, None),
                    name: "pushing".to_string(),
                },
                TokenType {
                    capacity: 0,
                    conditions: vec![Condition {
                        amount: 0,
                        object: ObjectSet::Object(tl_name.clone()),
                        temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                        value: "pushing".to_string(),
                    }],
                    duration: (1, None),
                    name: "moving".to_string(),
                },
                // TokenType {
                //     capacity: 0,
                //     conditions: vec![Condition {
                //         amount: 0,
                //         object: ObjectSet::Object(tl_name.clone()),
                //         temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                //         value: "moving".to_string(),
                //     }],
                //     duration: (1, None),
                //     name: "parked".to_string(),
                // },
                // TokenType {
                //     capacity: 0,
                //     conditions: vec![Condition {
                //         amount: 0,
                //         object: ObjectSet::Object(tl_name.clone()),
                //         temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                //         value: "moving".to_string(),
                //     }],
                //     duration: (1, None),
                //     name: "airborne".to_string(),
                // },
            ];
            timelines.insert(tl_name, values);
        }

        assert!(
            directions.iter().map(|d| d.as_str()).collect::<HashSet<_>>()
                == vec!["north", "south"].iter().copied().collect::<HashSet<_>>()
        );

        // RESOURCES have capacity 1 for airplanes.
        for segment in segments.iter() {
            for direction in directions.iter() {
                statictokens.push(Token {
                    capacity: 1,
                    const_time: TokenTime::Fact(None, None),
                    timeline_name: format!("occ_{}_{}", segment, direction),
                    value: "Available".to_string(),
                });
            }
        }

        // AIRPLANE PUSHBACK LOCATION
        for airplane in airplanes.iter() {
            let tl_name = format!("{}_pushback_loc", airplane);
            let mut values = Vec::new();

            let airplane_type = has_type.iter().find_map(|(a, t)| (airplane == a).then(|| t)).unwrap();

            for segment in segments.iter() {
                for direction in directions.iter() {
                    let location_name = format!("{}_{}", segment, direction);
                    let travel_time = 5; // TODO

                    let mut conditions = Vec::new();

                    // Need to use the current segment/dir and
                    // exclude all other uses of incompatible resources
                    conditions.push(Condition {
                        amount: 1,
                        object: ObjectSet::Object(format!("occ_{}_{}", segment, direction)),
                        temporal_relationship: TemporalRelationship::Cover,
                        value: "Available".to_string(),
                    });

                    let this_segment_blocks_other = is_blocked
                        .iter()
                        .filter(|(s1, t, _, _)| s1 == segment && t == airplane_type);

                    for (_, _, other_segment, other_dir) in this_segment_blocks_other {
                        conditions.push(Condition {
                            amount: 1,
                            object: ObjectSet::Object(format!("occ_{}_{}", other_segment, other_dir)),
                            temporal_relationship: TemporalRelationship::Cover,
                            value: "Available".to_string(),
                        });
                    }

                    values.push(TokenType {
                        capacity: 0,
                        conditions,
                        duration: (travel_time, None),
                        name: location_name.clone(),
                    });

                    values.push(TokenType {
                        capacity: 0,
                        conditions: vec![
                            Condition {
                                amount: 0,
                                object: ObjectSet::Object(tl_name.clone()),
                                temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                                value: location_name.clone(),
                            },
                            Condition {
                                amount: 0,
                                object: ObjectSet::Object(tl_name.clone()),
                                temporal_relationship: TemporalRelationship::Meets,
                                value: "pushback_finished".to_string(),
                            },
                            Condition {
                                amount: 0,
                                object: ObjectSet::Object(format!("{}_mode", airplane)),
                                temporal_relationship: TemporalRelationship::Meets,
                                value: "moving".to_string(),
                            },
                        ],
                        duration: (1, None),
                        name: format!("{}_{}_finishpushback", segment, direction),
                    })
                }
            }

            // CAN PUSHBACK: make connections between states

            for (from_seg, to_seg, from_dir) in can_pushback.iter() {
                let to_dir = move_back_dir
                    .iter()
                    .find_map(|(seg1, seg2, dir)| (seg1 == from_seg && seg2 == to_seg).then(|| dir))
                    .unwrap();

                let from_name = format!("{}_{}", from_seg, from_dir);
                let to_name = format!("{}_{}", to_seg, to_dir);

                values.push(TokenType {
                    capacity: 0,
                    conditions: vec![
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                            value: from_name,
                        },
                        Condition {
                            amount: 0,
                            object: ObjectSet::Object(tl_name.clone()),
                            temporal_relationship: TemporalRelationship::Meets,
                            value: to_name,
                        },
                    ],
                    duration: (1, None),
                    name: format!("{}_{}--{}_{}", from_seg, from_dir, to_seg, to_dir),
                });
            }

            values.push(TokenType {
                capacity: 0,
                conditions: vec![],
                duration: (1, None),
                name: "pushback_finished".to_string(),
            });

            timelines.insert(tl_name, values);
        }

        // AIRPLANE MOVING LOCATION
        for airplane in airplanes.iter() {
            let tl_name = format!("{}_moving_loc", airplane);
            let mut values = Vec::new();

            for segment in segments.iter() {
                for direction in directions.iter() {
                    let location_name = format!("{}_{}", segment, direction);
                    let travel_time = 5; // TODO

                    values.push(TokenType {
                        capacity: 0,
                        conditions: vec![],
                        duration: (travel_time, None),
                        name: location_name,
                    });
                }
            }

            values.push(TokenType {
                capacity: 0,
                conditions: vec![],
                duration: (1, None),
                name: "moving_finished".to_string(),
            });

            timelines.insert(tl_name, values);
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
        std::fs::write(&format!("airport_{}.json", file.file_name().to_str().unwrap()), &json).unwrap();
    }
}
