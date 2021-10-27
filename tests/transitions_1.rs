use timelinemodel::{problem::*, solver::solve};

#[test]
pub fn transitions_1() {
    let problem = Problem {
        groups: Vec::new(),
        timelines: vec![Timeline {
            name: "obj".to_string(),
            states: vec![
                State {
                    name: "s1".to_string(),
                    conditions: Vec::new(),
                    duration: (5, Some(6)),
                    capacity: 0,
                },
                State {
                    name: "s2".to_string(),
                    conditions: vec![
                        Condition {
                            temporal_relationship: TemporalRelationship::Meet,
                            amount: 0,
                            object: ObjectSet::Object("obj".to_string()),
                            value: "s1".to_string(),
                        }
                    ],
                    duration: (1, None),
                    capacity: 0,
                },
            ],
        }],
        tokens: vec![
            Token {timeline_name: "obj".to_string(), value: "s2".to_string(), const_time: TokenTime::Goal, capacity: 0 }
        ],
    };

    let solution = solve(&problem).unwrap();
println!("SOLUTION {:#?}", solution);
    assert!(solution.timelines.len() == 1);
    let timeline = &solution.timelines[0];
    assert!(timeline.name == "obj");
    assert!(timeline.tokens.len() == 2);
    let token1 = &timeline.tokens[1];
    let token2 = &timeline.tokens[0];
    assert!(token1.value == "s1");
    assert!(token2.value == "s2");
    assert!(token1.end_time - token1.start_time >= 5. && token1.end_time - token1.start_time <= 6.);
    assert!((token1.end_time - token2.start_time).abs() < 1e-5);
    assert!(token2.end_time.is_infinite());

}
