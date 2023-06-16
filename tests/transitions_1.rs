use paraspace::{problem::*, transitionsolver};

#[test]
pub fn transitions_1() {
    let problem = Problem {
        timelines: vec![Timeline {
            name: "obj".to_string(),
            token_types: vec![
                TokenType {
                    value: "s1".to_string(),
                    conditions: Vec::new(),
                    duration_limits: (5, Some(6)),
                    capacity: 0,
                },
                TokenType {
                    value: "s2".to_string(),
                    conditions: vec![vec![Condition {
                        temporal_relationship: TemporalRelationship::MetBy,
                        amount: 0,
                        timeline_ref: "obj".to_string(),
                        value: "s1".to_string(),
                    }]],
                    duration_limits: (1, None),
                    capacity: 0,
                },
            ],
            static_tokens: vec![Token {
                value: "s2".to_string(),
                const_time: TokenTime::Goal,
                capacity: 0,
                conditions: vec![],
            }],
        }],
    };

    println!("{}", serde_json::to_string(&problem).unwrap());

    let solution = transitionsolver::solve(&problem, &Default::default()).unwrap();
    println!("SOLUTION {:#?}", solution);
    assert!(solution.timelines.len() == 1);

    let token1 = &solution.timelines[0].tokens[0];
    let token2 = &solution.timelines[0].tokens[1];
    assert!(token1.value == "s1");
    assert!(token2.value == "s2");
    assert!(token1.end_time - token1.start_time >= 5. && token1.end_time - token1.start_time <= 6.);
    assert!((token1.end_time - token2.start_time).abs() < 1e-5);
    // assert!(token2.end_time.is_infinite());
}



#[test]
pub fn unrestricted_transitions() {
    let problem = Problem {
        timelines: vec![Timeline {
            name: "obj".to_string(),
            token_types: vec![
                TokenType {
                    value: "s1".to_string(),
                    conditions: Vec::new(),
                    duration_limits: (5, Some(6)),
                    capacity: 0,
                },
                TokenType {
                    value: "s2".to_string(),
                    conditions: vec![],
                    duration_limits: (1, None),
                    capacity: 0,
                },
            ],
            static_tokens: vec![Token {
                value: "s1".to_string(),
                const_time: TokenTime::Fact(Some(0), Some(5)),
                capacity: 0,
                conditions: vec![],
            },Token {
                value: "s2".to_string(),
                const_time: TokenTime::Goal,
                capacity: 0,
                conditions: vec![],
            }],
        }],
    };

    println!("{:#?}", problem);
    println!("{}", serde_json::to_string(&problem).unwrap());

    let solution = transitionsolver::solve(&problem, &Default::default()).unwrap();
    println!("SOLUTION {:#?}", solution);
    assert!(solution.timelines.len() == 1);

    let token1 = &solution.timelines[0].tokens[0];
    let token2 = &solution.timelines[0].tokens[1];   
     assert!(token1.value == "s1");
    assert!(token2.value == "s2");
    assert!(token1.end_time - token1.start_time >= 5. && token1.end_time - token1.start_time <= 6.);
    assert!((token1.end_time - token2.start_time).abs() < 1e-5);
    // assert!(token2.end_time.is_infinite());
}
