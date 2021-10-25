mod problem;
mod solver;

fn main() {
    println!("Hello, world!");

    let contents =std::fs::read_to_string("carbonara_5_problem.json").unwrap();
    let problem = serde_json::de::from_str::<problem::Problem>(&contents).unwrap();

    println!("Problem:\n{:#?}", problem);
    println!("Solving...");
    let solution = solver::solve(&problem);
    match solution {
        Ok(_) => {
            println!("Success!")
        }
        Err(err) => {
            println!("Error: {:#?}", err);
        }
    }
}



// Compilation idea:
//  Detect when two resources can be joined together into one
//  For example, in carbonara domain, boiling/cooking needs to select a plate,
//   and then use it exclusively, but if there are several plates they behave
//   just like if there was a resource with higher capacity. Symmetry reduction
//   effect by treating them as interchangable.