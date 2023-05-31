use paraspace::{problem, tokensolver};

// #[test]
pub fn carbonara5() {
    let problem = serde_json::de::from_str::<problem::Problem>(include_str!("carbonara_5_problem.json")).unwrap();

    println!("Problem:\n{:#?}", problem);
    println!("Solving...");
    let solution = tokensolver::solve(&problem, false);
    match solution {
        Ok(_) => {
            println!("Success!")
        }
        Err(err) => {
            println!("Error: {:#?}", err);
            panic!();
        }
    }
}