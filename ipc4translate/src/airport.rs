#![allow(clippy::vec_init_then_push)]
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
                    panic!("constant unexpected");
                }
            }
            ":durative-action" => {
                // // println!("ACTION");
                // let stmt = &stmt[1..];
                // for _ in stmt.iter() {
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
                                        // println!("LENGTH {:?}", stmt);
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
                                let expr = stmt[2].unwrap_list();

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

        let time_scale = 1000.0;

        assert!(
            directions.iter().map(|d| d.as_str()).collect::<HashSet<_>>()
                == vec!["north", "south"].iter().copied().collect::<HashSet<_>>()
        );

        // RESOURCES have capacity 1 for airplanes.
        for segment in segments.iter() {
            // for direction in directions.iter() {
            let occ_tl = format!("occupied_{}", segment);
            // let blk_tl = format!("blocked_{}", segment);

            timelines.insert(
                occ_tl.clone(),
                vec![
                    TokenType {
                        capacity: 0,
                        conditions: vec![Condition {
                            amount: 0,
                            object: ObjectSet::Object(occ_tl.clone()),
                            temporal_relationship: TemporalRelationship::MetBy,
                            value: "Yes".to_string(),
                        }],
                        duration: (1, None),
                        name: "No".to_string(),
                    },
                    TokenType {
                        capacity: 1,
                        conditions: vec![Condition {
                            amount: 0,
                            object: ObjectSet::Object(occ_tl.clone()),
                            temporal_relationship: TemporalRelationship::MetBy,
                            value: "No".to_string(),
                        }],
                        duration: (1, None),
                        name: "Yes".to_string(),
                    },
                ],
            );

            // timelines.insert(
            //     blk_tl.clone(),
            //     vec![
            //         TokenType {
            //             capacity: 0,
            //             conditions: vec![Condition {
            //                 amount: 0,
            //                 object: ObjectSet::Object(blk_tl.clone()),
            //                 temporal_relationship: TemporalRelationship::MetBy,
            //                 value: "Yes".to_string(),
            //             }],
            //             duration: (1, None),
            //             name: "No".to_string(),
            //         },
            //         TokenType {
            //             capacity: 0,
            //             conditions: vec![Condition {
            //                 amount: 0,
            //                 object: ObjectSet::Object(blk_tl.clone()),
            //                 temporal_relationship: TemporalRelationship::MetBy,
            //                 value: "No".to_string(),
            //             }],
            //             duration: (1, None),
            //             name: "Yes".to_string(),
            //         },
            //     ],
            // );

            statictokens.push(Token {
                capacity: 1,
                const_time: TokenTime::Fact(Some(0), None),
                timeline_name: format!("occupied_{}", segment),
                value: (if occupied.iter().any(|s| s == segment) {
                    "Yes"
                } else {
                    "No"
                })
                .to_string(),
                conditions: vec![],
            });

            // statictokens.push(Token {
            //     capacity: 1,
            //     const_time: TokenTime::Fact(Some(0), None),
            //     timeline_name: format!("blocked_{}", segment),
            //     value: (if blocked.iter().any(|(s, _airplane)| s == segment) {
            //         "Yes"
            //     } else {
            //         "No"
            //     })
            //     .to_string(),
            // });
            // }
        }

        // Assume that runways are unidirectional (simplifies the formulation of the airborne goals).
        for (a, b) in is_start_runway.iter() {
            for (c, d) in is_start_runway.iter() {
                if a == c {
                    assert!(b == d);
                }
            }
        }
        let push_mode = "Push";
        let move_mode = "Move";
        let park_mode = "Parked";
        let airborne_mode = "Airborne";

        for airplane in airplanes.iter() {
            // We'll make a graph of possible transitions for an airplane
            let initial_mode = if is_moving.iter().any(|a| airplane == a) {
                move_mode
            } else if is_pushing.iter().any(|a| airplane == a) {
                push_mode
            } else {
                assert!(airplane.contains("dummy"));
                continue;
            };

            let mut nodes = HashSet::new();
            let mut edges = Vec::new();

            // PUSHBACK EDGES
            for (from_seg, to_seg, from_dir) in can_pushback.iter() {
                let to_dir = move_back_dir
                    .iter()
                    .find_map(|(seg1, seg2, dir)| (seg1 == from_seg && seg2 == to_seg).then(|| dir))
                    .unwrap();

                let from_name = format!("{}_{}_{}", push_mode, from_seg, from_dir);
                let to_name = format!("{}_{}_{}", push_mode, to_seg, to_dir);
                nodes.insert((push_mode, from_seg, from_dir));
                nodes.insert((push_mode, to_seg, to_dir));
                edges.push((from_name, to_name));
            }
            // PUSH->MOVE edges
            for seg in segments.iter() {
                for dir in directions.iter() {
                    let from_name = format!("{}_{}_{}", push_mode, seg, dir);
                    let to_name = format!("{}_{}_{}", move_mode, seg, dir);
                    nodes.insert((push_mode, seg, dir));
                    nodes.insert((move_mode, seg, dir));
                    edges.push((from_name, to_name));
                }
            }
            // MOVE EDGES
            for (from_seg, to_seg, from_dir) in can_move.iter() {
                let to_dir = move_dir
                    .iter()
                    .find_map(|(seg1, seg2, dir)| (seg1 == from_seg && seg2 == to_seg).then(|| dir))
                    .unwrap();

                let from_name = format!("{}_{}_{}", move_mode, from_seg, from_dir);
                let to_name = format!("{}_{}_{}", move_mode, to_seg, to_dir);
                nodes.insert((move_mode, from_seg, from_dir));
                nodes.insert((move_mode, to_seg, to_dir));
                edges.push((from_name, to_name));
            }
            // PARK EDGES
            for seg in segments.iter() {
                for dir in directions.iter() {
                    let from_name = format!("{}_{}_{}", move_mode, seg, dir);
                    let to_name = format!("{}_{}", park_mode, seg);
                    nodes.insert((move_mode, seg, dir));
                    nodes.insert((park_mode, seg, dir));
                    edges.push((from_name, to_name));
                }
            }
            // TAKEOFF EDGES
            for (seg, dir) in is_start_runway.iter() {
                let from_name = format!("{}_{}_{}", move_mode, seg, dir);
                let to_name = format!("{}_{}_{}", airborne_mode, seg, dir);
                nodes.insert((move_mode, seg, dir));
                nodes.insert((airborne_mode, seg, dir));
                edges.push((from_name, to_name));
            }

            let mut values = Vec::new();
            let mut node_conditions: HashMap<String, HashMap<String, bool>> = HashMap::new();
            let mut node_travel_time: HashMap<String, usize> = HashMap::new();
            let mut added_nodes = HashSet::new();
            for (mode, seg, dir) in nodes {
                let name = if mode == park_mode {
                    format!("{}_{}", park_mode, seg)
                } else {
                    format!("{}_{}_{}", mode, seg, dir)
                };

                if !added_nodes.insert(name.clone()) {
                    continue;
                }

                let seg_length = length.iter().find_map(|(s, l)| (s == seg).then(|| *l)).unwrap();

                let travel_time = if mode == push_mode {
                    ((seg_length / 5.0) * time_scale + 0.5) as usize
                } else if mode == move_mode {
                    ((seg_length / 30.0) * time_scale + 0.5) as usize
                } else {
                    0
                };
                // println!("node time for name {} -> {}", name, travel_time);
                assert!(node_travel_time.insert(name.clone(), travel_time).is_none());

                let occupations = node_conditions.entry(name.clone()).or_default();

                if mode != airborne_mode {
                occupations.insert(format!("occupied_{}", seg), true);
            }

                // Block other resources
                if mode != park_mode && mode != airborne_mode {
                    for (other_seg, _airplanetype, this_seg, this_dir) in is_blocked.iter() {
                        if this_seg == seg && this_dir == dir {
                            // other_seg is blocked
                            occupations.entry(format!("occupied_{}", other_seg)).or_default();
                        }
                    }
                }

                values.push(TokenType {
                    capacity: 0,
                    conditions: occupations
                        .iter()
                        .map(|(occ, exclusive)| Condition {
                            amount: if *exclusive { 1 } else { 0 },
                            object: ObjectSet::Object(occ.clone()),
                            temporal_relationship: TemporalRelationship::Cover,
                            value: (if *exclusive { "Yes" } else { "No" }).to_string(),
                        })
                        .collect(),
                    duration: (1, None),
                    name,
                })
            }

            for (from, to) in edges {
                let mut conditions = vec![
                    Condition {
                        value: from.clone(),
                        amount: 0,
                        temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                        object: ObjectSet::Object(airplane.clone()),
                    },
                    Condition {
                        value: to.clone(),
                        amount: 0,
                        temporal_relationship: TemporalRelationship::Meets,
                        object: ObjectSet::Object(airplane.clone()),
                    },
                ];

                let mut occupations: HashMap<String, bool> = HashMap::new();
                for (occ, excl) in node_conditions[&from].iter().chain(node_conditions[&to].iter()) {
                    *occupations.entry(occ.clone()).or_default() |= *excl;
                }

                conditions.extend(occupations.iter().map(|(occ, exclusive)| Condition {
                    amount: if *exclusive { 1 } else { 0 },
                    object: ObjectSet::Object(occ.clone()),
                    temporal_relationship: TemporalRelationship::Cover,
                    value: (if *exclusive { "Yes" } else { "No" }).to_string(),
                }));

                let travel_time = node_travel_time[&from];

                let time = if from.starts_with(push_mode) && to.starts_with(move_mode) {
                    travel_time
                        + (60.
                            * engines
                                .iter()
                                .find_map(|(a, e)| (a == airplane).then(|| *e))
                                .unwrap_or(0.)
                            * time_scale
                            + 0.5) as usize
                } else if to.starts_with(park_mode) {
                    40 * (time_scale as usize)
                } else if to.starts_with(airborne_mode) {
                    30 * (time_scale as usize)
                } else {
                    travel_time
                };

                // println!("Total time {}->{} ===> {}", from, to, time);

                values.push(TokenType {
                    capacity: 0,
                    conditions,
                    duration: (time.max(1), None),
                    name: format!("{}->{}", from, to),
                })
            }

            timelines.insert(airplane.clone(), values);

            // INITIAL POSITIONS
            let initial_seg = at_segments
                .iter()
                .find_map(|(a, s)| (a == airplane).then(|| s))
                .unwrap();
            let initial_dir = facing.iter().find_map(|(a, d)| (a == airplane).then(|| d)).unwrap();

            statictokens.push(Token {
                capacity: 0,
                const_time: TokenTime::Fact(Some(0), None),
                timeline_name: airplane.clone(),
                value: format!("{}_{}_{}", initial_mode, initial_seg, initial_dir),
                conditions: node_conditions[&format!("{}_{}_{}", initial_mode, initial_seg, initial_dir)]
                    .iter()
                    .map(|(occ, exclusive)| Condition {
                        amount: if *exclusive { 1 } else { 0 },
                        object: ObjectSet::Object(occ.clone()),
                        temporal_relationship: TemporalRelationship::Cover,
                        value: (if *exclusive { "Yes" } else { "No" }).to_string(),
                    })
                    .collect(),
            });
        }

        //
        // GOALS
        //

        for (airplane, seg) in goal_parked.iter() {
            statictokens.push(Token {
                capacity: 0,
                const_time: TokenTime::Goal,
                timeline_name: airplane.to_string(),
                value: format!("{}_{}", park_mode, seg),
                conditions: vec![],
            });
        }

        for (airplane, seg) in goal_airborne.iter() {
            let dir = is_start_runway.iter().find_map(|(s, d)| (s == seg).then(|| d)).unwrap();
            statictokens.push(Token {
                capacity: 0,
                const_time: TokenTime::Goal,
                timeline_name: airplane.to_string(),
                value: format!("{}_{}_{}", airborne_mode, seg, dir),
                conditions: vec![],
            });
        }

        //
        // BLOCKED INTERVALS
        //
        let mut seg_blocked: HashMap<&String, Vec<(bool, usize)>> = HashMap::new();
        for (not, seg, plane, time) in blocked_intervals.iter() {
            assert!(plane.starts_with("dummy"));
            seg_blocked
                .entry(seg)
                .or_default()
                .push((*not, (time_scale * (*time) + 0.5) as usize));
        }

        for (seg, mut list) in seg_blocked {
            list.sort_by_key(|(_, t)| *t);
            assert!(list.len() % 2 == 0);

            while !list.is_empty() {
                let (n1, t1) = list.remove(0);
                let (n2, t2) = list.remove(0);
                assert!(!n1 && n2);

                // Blocked in interval t1 - t2

                statictokens.push(Token {
                    capacity: 0,
                    conditions: vec![Condition {
                        amount: 0,
                        object: ObjectSet::Object(format!("occupied_{}", seg.clone())),
                        temporal_relationship: TemporalRelationship::Cover,
                        value: "No".to_string(),
                    }],
                    const_time: TokenTime::Fact(Some(t1), Some(t2)),
                    timeline_name: "const_blocked".to_string(),
                    value: "Yes".to_string(),
                });
            }
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
