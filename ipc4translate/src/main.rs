mod airport;
mod pipesworld;
mod problem;
mod satellite;

fn main() {
    airport::convert_airport();
    satellite::convert_satellites();
    pipesworld::convert_pipesworld_notankage_temporal_deadlines();
    pipesworld::convert_pipesworld_batchencoding_notankage_temporal_deadlines();
}

trait SexpUnwrap {
    fn unwrap_atom(&self) -> &sexp::Atom;
    fn unwrap_list(&self) -> &Vec<sexp::Sexp>;
}

impl SexpUnwrap for sexp::Sexp {
    fn unwrap_atom(&self) -> &sexp::Atom {
        match self {
            sexp::Sexp::Atom(a) => a,
            _ => panic!(),
        }
    }

    fn unwrap_list(&self) -> &Vec<sexp::Sexp> {
        match self {
            sexp::Sexp::List(l) => l,
            _ => panic!(),
        }
    }
}
