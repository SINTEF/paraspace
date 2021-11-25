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
                        println!("obj types {}", objtype);
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
                            // "may-interface" => may_interface.push((
                            //     stmt[1].unwrap_atom().to_string().to_lowercase(),
                            //     stmt[2].unwrap_atom().to_string().to_lowercase(),
                            // )),
                            "=" => {
                                let lhs = stmt[1].unwrap_list();
                                let rhs = match stmt[2].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };

                                match lhs[0].unwrap_atom().to_string().to_lowercase().as_str() {
                                    // "speed" => {
                                    //     println!("Speed {}", stmt[1].to_string());
                                    //     speed.push((lhs[1].unwrap_atom().to_string().to_lowercase(), rhs));
                                    // }
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
                                let mut expr = stmt[2].unwrap_list();

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
                                            // "deliverable" => {
                                            //     not_deliverable
                                            //         .push((v[1].unwrap_atom().to_string().to_lowercase(), t));
                                            // }
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
                            // "on" => {
                            //     let a = goal[1].unwrap_atom().to_string().to_lowercase();
                            //     let b = goal[2].unwrap_atom().to_string().to_lowercase();
                            //     on_goal.push((a, b));
                            // }
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

        // let problem = Problem {
        //     groups: Vec::new(),
        //     timelines: timelines
        //         .into_iter()
        //         .map(|(n, v)| Timeline { name: n, values: v })
        //         .collect(),
        //     tokens: statictokens,
        // };
        // let json = serde_json::to_string(&problem).unwrap();
        // std::fs::write(
        //     &format!("pipesworld_{}.json", file.file_name().to_str().unwrap()),
        //     &json,
        // )
        // .unwrap();
    }
}
