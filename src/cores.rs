
pub fn minimize_core<'ctx>(core: &mut Vec<z3::ast::Bool<'ctx>>, solver: &z3::Solver<'ctx>) {
    println!("Starting core minimization.");
    let mut i = 0;
    'minimize_loop: loop {
        for _ in 0..core.len() {
            let last_core_size = core.len();
            let mut assumptions = core.clone();
            let remove_idx = i % assumptions.len();
            assumptions.remove(remove_idx);
            println!(
                "Solving core #{}->{} removed {}",
                core.len(),
                assumptions.len(),
                remove_idx
            );
            let result = solver.check_assumptions(&assumptions);
            if matches!(result, z3::SatResult::Unsat) {
                *core = solver.get_unsat_core();
                println!("Minimized {}->{}", last_core_size, core.len());
                continue 'minimize_loop;
            }
            i += 1;
        }
        println!("Finished core minimization.");
        break;
    }
}

pub fn trim_core<'ctx>(core: &mut Vec<z3::ast::Bool<'ctx>>, solver: &z3::Solver<'ctx>) {
    println!("Starting core trim.");
    loop {
        let last_core_size = core.len();
        // Try to trim the core.
        let result = solver.check_assumptions(&*core);
        assert!(matches!(result, z3::SatResult::Unsat));
        *core = solver.get_unsat_core();
        if core.len() == last_core_size {
            break;
        } else {
            println!("Trimmed {}->{}", last_core_size, core.len());
        }
    }
}