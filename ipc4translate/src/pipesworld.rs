use crate::{problem::*, SexpUnwrap};
use std::collections::{HashMap, HashSet};

/// Convert instance files for Pipesworld notankage temporal deadlines
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
                    let mut name = objs.remove(0).to_string();
                    while name != "-" {
                        names.push(name);
                        name = objs.remove(0).to_string();
                    }
                    let objtype = objs.remove(0);
                    println!("obj types {}", objtype);
                    match objtype.to_string().as_str() {
                        "product" => products.extend(names),
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
        assert!(stmts[0].unwrap_atom().to_string() == "define");
        assert!(stmts[1].unwrap_list()[0].to_string() == "problem");
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
            match stmt[0].unwrap_atom().to_string().as_str() {
                ":domain" => {
                    assert!(stmt[1].unwrap_atom().to_string() == domain_name);
                }
                ":objects" => {
                    let mut objs = stmt[1..].iter().collect::<Vec<_>>();
                    while objs.len() >= 3 {
                        let mut names = Vec::new();
                        let mut name = objs.remove(0).to_string();
                        while name != "-" {
                            names.push(name);
                            name = objs.remove(0).to_string();
                        }
                        let objtype = objs.remove(0);
                        println!("obj types {}", objtype);
                        match objtype.to_string().as_str() {
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
                            "unitary" => unitary.push((stmt[1].unwrap_atom().to_string().to_lowercase(),)),
                            "not-unitary" => not_unitary.push((stmt[1].unwrap_atom().to_string().to_lowercase(),)),
                            "=" => {
                                let lhs = stmt[1].unwrap_list();
                                let rhs = match stmt[2].unwrap_atom() {
                                    sexp::Atom::I(n) => *n as f64,
                                    sexp::Atom::F(n) => *n,
                                    _ => panic!(),
                                };

                                match lhs[0].unwrap_atom().to_string().as_str() {
                                    "speed" => {
                                        println!("Speed {}", stmt[1].to_string());
                                        speed.push((
                                            lhs[1].unwrap_atom().to_string().to_lowercase(),
                                            rhs,
                                        ));
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
                                        match v[0].unwrap_atom().to_string().as_str() {
                                            "deliverable" => {
                                                not_deliverable.push((v[1].unwrap_atom().to_string(), t));
                                            },
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


        // let mut batch_atoms = Vec::new();
        // let mut areas = Vec::new();
        // let mut pipes = Vec::new();

        // let mut deliverable = Vec::new();
        // let mut normal = Vec::new();
        // let mut may_interface = Vec::new();
        // let mut connect = Vec::new();
        // let mut is_product = Vec::new();
        // let mut on = Vec::new();
        
        // let mut first = Vec::new();
        // let mut last = Vec::new();
        // let mut follow = Vec::new();

        // let mut unitary = Vec::new();
        // let mut not_unitary = Vec::new();

        // let mut speed = Vec::new();
        // let mut on_goal = Vec::new();
        // let mut not_deliverable = Vec::new();



        // Problem interpretation
        // 



        // let problem = todo!();
        // let json = serde_json::to_string(&problem).unwrap();
        // std::fs::write(&format!("satellite_{}.json", file.file_name().to_str().unwrap()), &json).unwrap();
    }
}
