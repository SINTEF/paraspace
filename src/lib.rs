pub mod problem;
pub mod solver;
mod multiplicity;

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
