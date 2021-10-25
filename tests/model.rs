use timelinemodel::{problem::*, solver::solve};

// e2e simple models
#[test]
pub fn transitionfrom() {
    let problem = Problem {
        resources: vec![],
        timelines: vec![Timeline {
            class: "class".to_string(),
            name: Some("obj".to_string()),
            states: vec![
                State {
                    name: "s1".to_string(),
                    conditions: Vec::new(),
                    duration: (5, Some(6)),
                },
                State {
                    name: "s2".to_string(),
                    conditions: vec![Condition::TransitionFrom("s1".to_string())],
                    duration: (1, None),
                },
            ],
        }],
        facts: Vec::new(),
        goals: vec![
            TimelineValue { timeline_name: "obj".to_string(), value: "s2".to_string() }
        ]
    };

    let solution = solve(&problem).unwrap();

    assert!(solution.timelines.len() == 1);
    let timeline = &solution.timelines[0];
    assert!(timeline.name == "obj");
    assert!(timeline.class == "class");
    assert!(timeline.tokens.len() == 2);
    let token1 = &timeline.tokens[1];
    let token2 = &timeline.tokens[0];
    assert!(token1.value == "s1");
    assert!(token2.value == "s2");
    assert!(token1.end_time - token1.start_time >= 5. && token1.end_time - token1.start_time <= 6.);
    assert!((token1.end_time - token2.start_time).abs() < 1e-5);
    assert!(token2.end_time.is_infinite());

}
