pub mod problem;
pub mod transitionsolver;
pub mod cores;

pub fn solve_json(input :String) -> String {
    let problem = serde_json::de::from_str::<problem::Problem>(&input).unwrap();
    println!("{:#?}", problem);
    "".to_string()
}

pub fn to_json(input :&problem::Problem) -> String {
    serde_json::to_string_pretty(input).unwrap()
}

pub fn print_calc_time<T>(name: &str, f: impl FnOnce() -> T) -> T{
    use std::time::Instant;
    let now = Instant::now();

    let result = {
        f()
    };

    let elapsed = now.elapsed();
    println!("{} took {:.2?}", name, elapsed);
    result
}

#[derive(Clone, Debug)]
pub enum SolverError {
    NoSolution,
    GoalValueDurationLimit,
    GoalStateMissing,
}


pub fn z3real_value(real: &z3::ast::Real) -> f32 {
    let (num, den) = real.as_real().unwrap();
    num as f32 / den as f32
}
