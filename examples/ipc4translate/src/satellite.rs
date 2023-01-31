use crate::{problem::*, SexpUnwrap};
use std::collections::{HashMap, HashSet};

pub fn convert_satellites() {
    let dir = "inputs/satellite";
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
        assert!(stmts[0].unwrap_atom().to_string() == "define");
        assert!(stmts[1].unwrap_list()[0].to_string() == "problem");
        let problem_name = stmts[1].unwrap_list()[1].to_string();
        println!("Problem name: {}", problem_name);

        // let mut objects = Vec::new();

        let mut satellites = Vec::new();
        let mut instruments = Vec::new();
        let mut modes = Vec::new();
        let mut directions = Vec::new();
        let mut antennas = Vec::new();

        let mut available_antennas = Vec::new();

        let mut supports = Vec::new();
        let mut calibration_target = Vec::new();
        let mut calibration_time = Vec::new();
        let mut on_board = Vec::new();
        let mut power_avail = Vec::new();
        let mut pointing = Vec::new();
        let mut slew_time = Vec::new();
        let mut send_time = Vec::new();

        let mut visibility_windows = HashSet::new();

        let mut goal_images = Vec::new();
        let mut goal_pointing = Vec::new();

        for stmt in stmts[2..].iter() {
            let stmt = stmt.unwrap_list();
            match stmt[0].unwrap_atom().to_string().as_str() {
                ":domain" => {
                    assert!(stmt[1].unwrap_atom().to_string() == domain_name);
                }
                ":objects" => {
                    let mut objs = stmt[1..].iter().collect::<Vec<_>>();
                    while objs.len() >= 3 {
                        let name = objs.remove(0);
                        let dash = objs.remove(0);
                        assert!(dash.to_string() == "-");
                        let objtype = objs.remove(0);
                        // objects.push((name, objtype));

                        match objtype.to_string().as_str() {
                            "satellite" => satellites.push(name.to_string().to_lowercase()),
                            "instrument" => instruments.push(name.to_string().to_lowercase()),
                            "mode" => modes.push(name.to_string().to_lowercase()),
                            "direction" => directions.push(name.to_string().to_lowercase()),
                            "antenna" => antennas.push(name.to_string().to_lowercase()),
                            _ => panic!(),
                        }
                    }
                }
                ":init" => {
                    for initstmt in &stmt[1..] {
                        let stmt = initstmt.unwrap_list();
                        match stmt[0].unwrap_atom().to_string().to_lowercase().as_str() {
                            "supports" => supports.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "calibration_target" => calibration_target.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "on_board" => on_board.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "power_avail" => power_avail.push(stmt[1].unwrap_atom().to_string().to_lowercase()),
                            "pointing" => pointing.push((
                                stmt[1].unwrap_atom().to_string().to_lowercase(),
                                stmt[2].unwrap_atom().to_string().to_lowercase(),
                            )),
                            "=" => {
                                let lhs = stmt[1].unwrap_list();
                                let rhs = match stmt[2].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };

                                match lhs[0].unwrap_atom().to_string().as_str() {
                                    "slew_time" => {
                                        slew_time.push((
                                            lhs[1].unwrap_atom().to_string().to_lowercase(),
                                            lhs[2].unwrap_atom().to_string().to_lowercase(),
                                            rhs,
                                        ));
                                    }
                                    "calibration_time" => {
                                        calibration_time.push((
                                            lhs[1].unwrap_atom().to_string().to_lowercase(),
                                            lhs[2].unwrap_atom().to_string().to_lowercase(),
                                            rhs,
                                        ));
                                    }
                                    "send_time" => {
                                        send_time.push((
                                            lhs[1].unwrap_atom().to_string().to_lowercase(),
                                            lhs[2].unwrap_atom().to_string().to_lowercase(),
                                            rhs,
                                        ));
                                    }
                                    _ => panic!(),
                                };
                            }
                            "at" => {
                                let t = match stmt[1].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };
                                let mut expr = stmt[2].unwrap_list();

                                let not = if expr[0].unwrap_atom().to_string().as_str().to_lowercase() == "not" {
                                    expr = expr[1].unwrap_list();
                                    true
                                } else {
                                    false
                                };

                                match expr[0].unwrap_atom().to_string().as_str() {
                                    "visible" => {
                                        let antenna = expr[1].unwrap_atom().to_string().to_lowercase();

                                        // The satellite conditions seems to not be used in the original
                                        // PDDL problem.
                                        let _satellite = expr[2].unwrap_atom().to_string().to_lowercase();

                                        let t = (t * 100. + 0.5) as usize;
                                        // let t2 = (t2 * 100. + 0.5) as usize;

                                        visibility_windows.insert((antenna, t, not));
                                    }
                                    _ => panic!(),
                                };
                            }
                            "available" => {
                                let a = stmt[1].unwrap_atom().to_string().to_lowercase();
                                if antennas.iter().any(|b| &a == b) {
                                    available_antennas.push(a);
                                } else {
                                    panic!("available what?");
                                }
                            }

                            _ => panic!(),
                        }
                    }
                }
                ":goal" => {
                    let goals = stmt[1].unwrap_list();
                    assert!(goals[0].unwrap_atom().to_string().as_str() == "and");
                    for goal in goals[1..].iter() {
                        let goal = goal.unwrap_list();
                        match goal[0].unwrap_atom().to_string().as_str() {
                            "sent_image" => {
                                let a = goal[1].unwrap_atom().to_string().to_lowercase();
                                let b = goal[2].unwrap_atom().to_string().to_lowercase();
                                goal_images.push((a, b));
                            }
                            "pointing" => {
                                let a = goal[1].unwrap_atom().to_string().to_lowercase();
                                let b = goal[2].unwrap_atom().to_string().to_lowercase();
                                goal_pointing.push((a, b));
                            }
                            g => {
                                println!("goal {}", g);
                                panic!()
                            }
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

        // Antennas are available
        assert!(antennas.iter().all(|a| available_antennas.iter().any(|b| a == b)));
        // Satellites have power
        assert!(satellites.iter().all(|s| power_avail.iter().any(|b| s == b)));
        // INSTRUMENTS are many-to-one with SATELLITES
        let mut instrument_belongs_to = HashMap::new();
        for (i, s) in on_board.iter() {
            assert!(instrument_belongs_to.insert(i, s).is_none());
        }

        // let mut satellites = Vec::new();
        // let mut instruments = Vec::new();
        // let mut modes = Vec::new();
        // let mut directions = Vec::new();
        // let mut antennas = Vec::new();

        // let mut available_antennas = Vec::new();

        // let mut supports = Vec::new();
        // let mut calibration_target = Vec::new();
        // let mut calibration_time = Vec::new();
        // let mut on_board = Vec::new();
        // let mut power_avail = Vec::new();
        // let mut pointing = Vec::new();
        // let mut slew_time = Vec::new();
        // let mut send_time = Vec::new();

        // let mut visibility_windows = Vec::new();

        // let mut goal_images = Vec::new();
        // let mut goal_pointing = Vec::new();

        // TIMELINES
        let mut timelines: HashMap<String, Vec<TokenType>> = HashMap::new();
        let mut statictokens = Vec::new();

        // 1.
        //
        // SATELITE DIRECTION POINTING
        //
        //
        for satellite in satellites.iter() {
            let timeline_name = format!("dir_{}", satellite);
            let mut tokentypes = Vec::new();

            for dir in directions.iter() {
                tokentypes.push(TokenType {
                    name: dir.to_string(),
                    duration: (1, None),
                    conditions: vec![],
                    capacity: 0,
                });
            }

            for (a, b, l) in slew_time.iter() {
                let dur = (l * 100.0 + 0.5) as usize;
                tokentypes.push(TokenType {
                    name: format!("slew_{}_{}", a, b),
                    duration: (dur, Some(dur)),
                    conditions: vec![
                        Condition {
                            object: ObjectSet::Object(timeline_name.clone()),
                            value: a.clone(),
                            temporal_relationship: TemporalRelationship::MetBy,
                            amount: 0,
                        },
                        Condition {
                            object: ObjectSet::Object(timeline_name.clone()),
                            value: b.clone(),
                            temporal_relationship: TemporalRelationship::Meets,
                            amount: 0,
                        },
                    ],
                    capacity: 0,
                });
            }

            timelines.insert(timeline_name, tokentypes);
        }
        println!(
            "{} satellites X {} slews = {}",
            satellites.len(),
            slew_time.len(),
            timelines.len()
        );

        // // 2. HAVEIMAGE
        // for dir in directions.iter() {
        //     for mode in modes.iter() {
        //         // Is there an instrument that takes this picture?
        //         let tl_name = format!("HaveImage_{}_{}", dir, mode);
        //         timelines.insert(
        //             tl_name,
        //             vec![TokenType {
        //                 name: "Yes".to_string(),
        //                 duration: (1, None),
        //                 conditions: vec![Condition {
        //                     temporal_relationship: TemporalRelationship::MetBy,
        //                     amount: 0,
        //                     object: ObjectSet::Group("Instruments".to_string()),
        //                     value: format!("TakeImage_{}_{}", dir, mode),
        //                 }],
        //                 capacity: 0,
        //             }],
        //         );
        //     }
        // }

        println!("haveimage dirs = {} X modes = {}", directions.len(), modes.len());

        //
        // 2. INSTRUMENT TAKEIMAGE MODE DIRECTION
        //
        //
        for instrument in instruments.iter() {
            let tl_name = instrument;
            let power_cond = Condition {
                amount: 0,
                object: ObjectSet::Object(format!("power_instrument_{}", instrument)),
                value: "Available".to_string(),
                temporal_relationship: TemporalRelationship::Cover,
            };

            statictokens.push(Token {
                capacity: 0,
                conditions: vec![],
                const_time: TokenTime::Fact(None,None),
                timeline_name: format!("power_instrument_{}", instrument),
                value: "NotAvailable".to_string()
            });

            timelines.insert(
                format!("power_instrument_{}", instrument),
                vec![TokenType {
                    capacity: 0,
                    conditions: vec![Condition {
                        amount: 1,
                        object: ObjectSet::Object(format!("power_{}", instrument_belongs_to[instrument])),
                        value: "Available".to_string(),
                        temporal_relationship: TemporalRelationship::Cover,
                    },
                    Condition {
                        amount: 0,
                        object: ObjectSet::Object(format!("power_instrument_{}", instrument)),
                        value: "NotAvailable".to_string(),
                        temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                    }],
                    duration: (1, None),
                    name: "Available".to_string(),
                },
                TokenType {
                    capacity: 0,
                    conditions: vec![Condition {
                        amount: 0,
                        object: ObjectSet::Object(format!("power_instrument_{}", instrument)),
                        value: "Available".to_string(),
                        temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                    }],
                    duration: (1, None),
                    name: "NotAvailable".to_string(),
                }],
            );

            statictokens.push(Token {
                timeline_name: tl_name.clone(),
                value: "Off".to_string(),
                capacity: 0,
                const_time: TokenTime::Fact(None, None),
                conditions: vec![],

            });

            let mut values = vec![
                TokenType {
                    name: "Off".to_string(),
                    capacity: 0,
                    conditions: vec![],
                    duration: (1, None),
                },
                TokenType {
                    name: "SwitchOn".to_string(),
                    capacity: 0,
                    conditions: vec![
                        Condition {
                            temporal_relationship: TemporalRelationship::MetBy,
                            object: ObjectSet::Object(tl_name.clone()),
                            amount: 0,
                            value: "Off".to_string(),
                        },
                        power_cond.clone(),
                    ],
                    duration: (200, None),
                },
                TokenType {
                    name: "SwitchOff".to_string(),
                    capacity: 0,
                    conditions: vec![
                        Condition {
                            temporal_relationship: TemporalRelationship::Meets,
                            object: ObjectSet::Object(tl_name.clone()),
                            amount: 0,
                            value: "Off".to_string(),
                        },
                        Condition {
                            temporal_relationship: TemporalRelationship::MetBy,
                            object: ObjectSet::Object(tl_name.clone()),
                            amount: 0,
                            value: "Calibrated".to_string(),
                        },
                        power_cond.clone(),
                    ],
                    duration: (100, None),
                },
                TokenType {
                    name: "Calibrated".to_string(),
                    capacity: 0,
                    conditions: vec![power_cond.clone()],
                    duration: (1, None),
                },
            ];

            for dir in directions.iter() {
                if calibration_target.iter().any(|(i, d)| i == instrument && d == dir) {
                    let t = *calibration_time
                        .iter()
                        .find_map(|(i, d, t)| (i == instrument && d == dir).then(|| t))
                        .unwrap();
                    let dur = (t * 100. + 0.5) as usize;
                    values.push(TokenType {
                        name: format!("calibrate_{}", dir),
                        duration: (dur, None),
                        capacity: 0,
                        conditions: vec![
                            Condition {
                                temporal_relationship: TemporalRelationship::MetBy,
                                object: ObjectSet::Object(tl_name.clone()),
                                amount: 0,
                                value: "SwitchOn".to_string(),
                            },
                            Condition {
                                temporal_relationship: TemporalRelationship::Meets,
                                object: ObjectSet::Object(tl_name.clone()),
                                amount: 0,
                                value: "Calibrated".to_string(),
                            },
                            power_cond.clone(),
                        ],
                    });
                }
            }

            for dir in directions.iter() {
                for mode in modes.iter() {
                    if supports.iter().any(|(i, m)| i == instrument && m == mode) {
                        values.push(TokenType {
                            name: format!("TakeImage_{}_{}", dir, mode),
                            duration: (700, None),
                            capacity: 0,
                            conditions: vec![
                                Condition {
                                    temporal_relationship: TemporalRelationship::MetBy,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    amount: 0,
                                    value: "Calibrated".to_string(),
                                },
                                Condition {
                                    temporal_relationship: TemporalRelationship::Meets,
                                    object: ObjectSet::Object(tl_name.clone()),
                                    amount: 0,
                                    value: "Calibrated".to_string(),
                                },
                                Condition {
                                    temporal_relationship: TemporalRelationship::Cover,
                                    object: ObjectSet::Object(format!("dir_{}", instrument_belongs_to[instrument])),
                                    amount: 0,
                                    value: dir.clone(),
                                },
                                power_cond.clone(),
                            ],
                        });
                    }
                }
            }

            timelines.insert(tl_name.clone(), values);
        }

        println!(
            "instruments= {} dirs = {} X modes = {}",
            instruments.len(),
            directions.len(),
            modes.len()
        );

        //
        // 3.
        //
        // SEND IMAGES
        for (dir, mode) in goal_images.iter() {
            let t = send_time
                .iter()
                .find_map(|(d, m, t)| (d == dir && m == mode).then(|| *t))
                .unwrap();
            let dur = (t * 100. + 0.5) as usize;

            let timeline_name = format!("SendImage_{}_{}", dir, mode);
            let tokentypes = vec![
                TokenType {
                    name: "HaveImage".to_string(),
                    duration: (dur, Some(dur)),
                    conditions: vec![Condition {
                        temporal_relationship: TemporalRelationship::MetBy,
                        amount: 0,
                        object: ObjectSet::Group("Instruments".to_string()),
                        value: format!("TakeImage_{}_{}", dir, mode),
                    }],
                    capacity: 0,
                },
                TokenType {
                    name: "Send".to_string(),
                    duration: (dur, Some(dur)),
                    conditions: vec![
                        Condition {
                            temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                            object: ObjectSet::Object(timeline_name.clone()),
                            amount: 0,
                            value: "HaveImage".to_string(),
                        },
                        Condition {
                            temporal_relationship: TemporalRelationship::Cover,
                            object: ObjectSet::Group("Antennas".to_string()),
                            amount: 1,
                            value: "Available".to_string(),
                        },
                    ],
                    capacity: 0,
                },
                TokenType {
                    name: "Sent".to_string(),
                    duration: (1, None),
                    conditions: vec![Condition {
                        temporal_relationship: TemporalRelationship::MetByTransitionFrom,
                        object: ObjectSet::Object(timeline_name.clone()),
                        amount: 0,
                        value: "Send".to_string(),
                    }],
                    capacity: 0,
                },
            ];

            timelines.insert(timeline_name.clone(), tokentypes);

            statictokens.push(Token {
                timeline_name,
                capacity: 0,
                const_time: TokenTime::Goal,
                value: "Sent".to_string(),
                conditions: vec![],

            });
        }

        // SATELLITE POWER
        for sat in satellites.iter() {
            statictokens.push(Token {
                timeline_name: format!("power_{}", sat),
                value: "Available".to_string(),
                capacity: 1,
                const_time: TokenTime::Fact(None, None),
                conditions: vec![],

            });
        }

        // POINTSTO facts
        for (sat, dir) in pointing.iter() {
            let timeline_name = format!("dir_{}", sat);
            let value = dir.clone();
            statictokens.push(Token {
                timeline_name,
                value,
                capacity: 0,
                const_time: TokenTime::Fact(None, None),
                conditions: vec![],

            });
        }

        // POINTSTO goal
        for (sat, dir) in goal_pointing.iter() {
            let timeline_name = format!("dir_{}", sat);
            let value = dir.clone();
            statictokens.push(Token {
                timeline_name,
                value,
                capacity: 0,
                const_time: TokenTime::Goal,
                                conditions: vec![],

            });
        }

        // ANTENNAS VISIBLE TIME WINDOWS

        let mut antennas: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
        let mut start_times = HashMap::new();

        let mut visibility_windows = visibility_windows.into_iter().collect::<Vec<_>>();
        visibility_windows.sort();

        for (a, t, on) in visibility_windows.iter() {
            if !on {
                assert!(start_times.insert(a, t).is_none());
            } else {
                let t0 = start_times.remove(&a).unwrap();
                antennas.entry(a.clone()).or_default().push((*t0, *t));
            }
        }
        for (antenna, windows) in antennas.iter() {
            for (t1, t2) in windows {
                statictokens.push(Token {
                    timeline_name: antenna.clone(),
                    value: "Available".to_string(),
                    capacity: 1,
                    const_time: TokenTime::Fact(Some(*t1), Some(*t2)),
                    conditions: vec![],

                })
            }
        }

        let groups = vec![
            Group {
                name: "Instruments".to_string(),
                members: instruments.to_vec(),
            },
            Group {
                name: "Antennas".to_string(),
                members: antennas.keys().cloned().collect(),
            },
        ];

        let problem = Problem {
            groups,
            timelines: timelines
                .into_iter()
                .map(|(n, t)| Timeline { name: n, values: t })
                .collect(),
            tokens: statictokens,
        };

        let json = serde_json::to_string(&problem).unwrap();
        std::fs::write(&format!("satellite_{}.json", file.file_name().to_str().unwrap()), &json).unwrap();
    }
}
