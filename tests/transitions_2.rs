use timelinemodel::{problem::*, tokensolver::solve};

#[test]
pub fn transitions_2() {
    let problem = Problem {
        groups: Vec::new(),
        timelines: vec![Timeline {
            name: "obj".to_string(),
            values: vec![
                Value {
                    name: "s1".to_string(),
                    conditions: Vec::new(),
                    duration: (5, Some(6)),
                    capacity: 0,
                },
                Value {
                    name: "s2".to_string(),
                    conditions: vec![Condition {
                        temporal_relationship: TemporalRelationship::MetBy,
                        amount: 0,
                        object: ObjectSet::Object("obj".to_string()),
                        value: "s1".to_string(),
                    }],
                    duration: (1, None),
                    capacity: 0,
                },
                Value {
                    name: "s3".to_string(),
                    conditions: vec![Condition {
                        temporal_relationship: TemporalRelationship::MetBy,
                        amount: 0,
                        object: ObjectSet::Object("obj".to_string()),
                        value: "s2".to_string(),
                    }],
                    duration: (1, None),
                    capacity: 0,
                },
            ],
        }],
        tokens: vec![Token {
            timeline_name: "obj".to_string(),
            value: "s3".to_string(),
            const_time: TokenTime::Goal,
            capacity: 0,
        }],
    };

    let solution = solve(&problem).unwrap();
    println!("SOLUTION {:#?}", solution);

    assert!(solution.tokens.len() == 3);

    let token0 = &solution.tokens[2];
    let token1 = &solution.tokens[1];
    let token2 = &solution.tokens[0];

    assert!(token0.value == "s1");
    assert!(token1.value == "s2");
    assert!(token2.value == "s3");
    assert!(token0.end_time - token0.start_time >= 5. && token0.end_time - token0.start_time <= 6.);
    assert!((token1.end_time - token2.start_time).abs() < 1e-5);
    assert!((token0.end_time - token1.start_time).abs() < 1e-5);
    assert!(token2.end_time.is_infinite());
}
