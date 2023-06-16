use paraspace::{problem::*, transitionsolver::solve};

#[test]
pub fn transitions_2() {
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
                TokenType {
                    value: "s3".to_string(),
                    conditions: vec![vec![Condition {
                        temporal_relationship: TemporalRelationship::MetBy,
                        amount: 0,
                        timeline_ref: "obj".to_string(),
                        value: "s2".to_string(),
                    }]],
                    duration_limits: (1, None),
                    capacity: 0,
                },
            ],
            static_tokens: vec![Token {
                value: "s3".to_string(),
                const_time: TokenTime::Goal,
                capacity: 0,
                conditions: vec![],
            }],
        }],
    };

    let solution = solve(&problem, &Default::default()).unwrap();
    println!("SOLUTION {:#?}", solution);

    assert!(solution.timelines.len() == 1);

    let token0 = &solution.timelines[0].tokens[0];
    let token1 = &solution.timelines[0].tokens[1];
    let token2 = &solution.timelines[0].tokens[2];   

    assert!(token0.value == "s1");
    assert!(token1.value == "s2");
    assert!(token2.value == "s3");
    assert!(token0.end_time - token0.start_time >= 5. && token0.end_time - token0.start_time <= 6.);
    assert!((token1.end_time - token2.start_time).abs() < 1e-5);
    assert!((token0.end_time - token1.start_time).abs() < 1e-5);
    assert!(token2.end_time == solution.end_of_time);
}
