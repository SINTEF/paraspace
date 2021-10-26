use std::path::PathBuf;

use structopt::StructOpt;
use timelinemodel::print_calc_time;

mod multiplicity;
mod problem;
mod solver;

#[derive(Debug, StructOpt)]
#[structopt(name = "timelinemodel", about = "Timelines SMT-based solver.")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(long = "benchmark")]
    perftest: bool,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    if opt.perftest {
        perftest();
    }

    if let Some(filename) = opt.input {
        let contents = std::fs::read_to_string(&filename).unwrap();
        let problem = serde_json::de::from_str::<problem::Problem>(&contents).unwrap();
        let result = print_calc_time(filename.to_str().unwrap(), || solver::solve(&problem));
        match result {
            Ok(solution) => {
                println!("Solved.");

                if let Some(output) = opt.output {
                    std::fs::write(&output, serde_json::to_string_pretty(&solution).unwrap()).unwrap();

                    println!("Wrote to file '{}'", output.to_str().unwrap());
                }
            }
            Err(err) => {
                println!("Error: {:#?}", err);
            }
        }
    } else {
        println!("No problem files given.");
    }
}

fn perftest() {
    for plates in [1, 2] {
        for n_carbonaras in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20, 25, 30, 40, 50, 75, 100] {
            let problem_name = format!("carbonara_{}p_{}c", plates, n_carbonaras);
            let contents = std::fs::read_to_string(&format!("examples/{}.json", problem_name)).unwrap();
            let problem = serde_json::de::from_str::<problem::Problem>(&contents).unwrap();

            // println!("Problem:\n{:#?}", problem);
            // println!("Solving...");
            let result = print_calc_time(&problem_name, || solver::solve(&problem));
            match result {
                Ok(solution) => {
                    // println!("Success!");
                    std::fs::write(
                        &format!("examples/{}.out.json", problem_name),
                        serde_json::to_string_pretty(&solution).unwrap(),
                    )
                    .unwrap();
                }
                Err(err) => {
                    println!("Error: {:#?}", err);
                }
            }
        }
    }
}

// Compilation idea:
//  Detect when two resources can be joined together into one
//  For example, in carbonara domain, boiling/cooking needs to select a plate,
//   and then use it exclusively, but if there are several plates they behave
//   just like if there was a resource with higher capacity. Symmetry reduction
//   effect by treating them as interchangable.
