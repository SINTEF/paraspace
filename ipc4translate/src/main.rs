mod problem;
mod satellite;
mod pipesworld;
mod airport;

use std::collections::{HashMap, HashSet};

use sexp::Sexp;

use crate::problem::*;

fn main() {
    airport::convert_airport();
    // satellite::convert_satellites();
    // pipesworld::convert_pipesworld_notankage_temporal_deadlines();
}

trait SexpUnwrap {
    fn unwrap_atom(&self) -> &sexp::Atom;
    fn unwrap_list(&self) -> &Vec<sexp::Sexp>;
}

impl SexpUnwrap for sexp::Sexp {
    fn unwrap_atom(&self) -> &sexp::Atom {
        match self {
            Sexp::Atom(a) => a,
            _ => panic!(),
        }
    }

    fn unwrap_list(&self) -> &Vec<sexp::Sexp> {
        match self {
            Sexp::List(l) => l,
            _ => panic!(),
        }
    }
}
