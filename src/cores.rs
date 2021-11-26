
pub fn minimize_core<'ctx>(core: &mut Vec<z3::ast::Bool<'ctx>>, solver: &z3::Solver<'ctx>, print :impl Fn(&str)) {
    print("Starting core minimization.");
    let mut i = 0;
    'minimize_loop: loop {
        for _ in 0..core.len() {
            let last_core_size = core.len();
            let mut assumptions = core.clone();
            let remove_idx = i % assumptions.len();
            assumptions.remove(remove_idx);
            print(&format!(
                "Solving core #{}->{} removed {}",
                core.len(),
                assumptions.len(),
                remove_idx
            ));
            let result = solver.check_assumptions(&assumptions);
            if matches!(result, z3::SatResult::Unsat) {
                *core = solver.get_unsat_core();
                print(&format!("Minimized {}->{}", last_core_size, core.len()));
                continue 'minimize_loop;
            }
            i += 1;
        }
        print(&"Finished core minimization.".to_string());
        break;
    }
}

pub fn trim_core<'ctx>(core: &mut Vec<z3::ast::Bool<'ctx>>, solver: &z3::Solver<'ctx>, print :impl Fn(&str)) {
    print("Starting core trim.");
    loop {
        let last_core_size = core.len();
        // Try to trim the core.
        let result = solver.check_assumptions(&*core);
        assert!(matches!(result, z3::SatResult::Unsat));
        *core = solver.get_unsat_core();
        if core.len() == last_core_size {
            break;
        } else {
            print(&format!("Trimmed {}->{}", last_core_size, core.len()));
        }
    }
}