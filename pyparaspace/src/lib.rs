use paraspace::{problem::*, transitionsolver::solve};
use pyo3::prelude::*;

#[pyclass(name = "Problem")]
pub struct ProblemPy {
    #[pyo3(get)]
    pub timelines: Vec<TimelinePy>,
    #[pyo3(get)]
    pub groups: Vec<GroupPy>,
    #[pyo3(get)]
    pub tokens: Vec<TokenPy>
}

#[pyclass(name = "Timeline")]
#[derive(Clone)]
pub struct TimelinePy {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub values: Vec<ValuePy>,
}

#[pyclass(name = "Value")]
#[derive(Clone)]
pub struct ValuePy {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub duration: (usize, Option<usize>),
    #[pyo3(get)]
    pub conditions: Vec<ConditionPy>,
    #[pyo3(get)]
    pub capacity: u32,
}

#[pyclass(name = "Condition")]
#[derive(Clone)]
pub struct ConditionPy {
    #[pyo3(get)]
    pub temporal_relationship: TemporalRelationshipPy,
    #[pyo3(get)]
    pub object: Vec<String>,
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub amount: u32,
}

#[pyclass(name = "TemporalRelationship")]
#[derive(Clone)]
pub enum TemporalRelationshipPy {
    MetBy,
    MetByTransitionFrom,
    Meets,
    Cover,
    Equal,
    StartsAfter,
}

#[pyclass(name = "Group")]
#[derive(Clone)]
pub struct GroupPy {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub members: Vec<String>,
}

type TokenTimePy = Option<(Option<usize>,Option<usize>)>;

#[pyclass(name = "Token")]
#[derive(Clone)]
pub struct TokenPy {
    #[pyo3(get)]
    pub timeline_name: String,
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub capacity: u32,
    #[pyo3(get)]
    pub const_time: TokenTimePy,
    #[pyo3(get)]
    pub conditions: Vec<ConditionPy>,
}

//
// SOLUTION
//

#[pyclass(name = "Solution")]
#[derive(Clone)]
pub struct SolutionPy {
    #[pyo3(get)]
    pub tokens: Vec<SolutionTokenPy>,
}

#[pyclass(name = "SolutionToken")]
#[derive(Clone)]
pub struct SolutionTokenPy {
    #[pyo3(get)]
    pub object_name: String,
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub start_time: f32,
    #[pyo3(get)]
    pub end_time: f32,
}

#[pymethods]
impl ProblemPy {
    #[new]
    fn init(timelines: Vec<TimelinePy>, groups: Vec<GroupPy>, tokens: Vec<TokenPy>) -> Self {
        ProblemPy { timelines, groups, tokens }
    }

    fn __repr__(&self) -> String {
        format!("Problem({} groups)", self.groups.len())
    }
}

#[pymethods]
impl TimelinePy {
    #[new]
    fn init(name: String, values: Vec<ValuePy>) -> Self {
        TimelinePy {name, values}
    }

    fn __repr__(&self) -> String {
        format!("Timeline(name: {}, values: ({}))", self.name, self.values)
    }
}

#[pymethods]
impl ValuePy {
    #[new]
    fn init(name: String, duration: (usize, Option<usize>), conditions: Vec<ConditionPy>, capacity: u32) -> Self {
        ValuePy {name, duration, conditions, capacity}
    }

    fn __repr__(&self) -> String {
        format!("Value(name: {}, duration: {}, conditions: (), capacity)", self.name, self.duration, self.conditions, self.capacity)
    }
}

#[pymethods]
impl ConditionPy {
    #[new]
    fn init(temporal_relationship: TemporalRelationshipPy, object: Vec<String>, value: String, amount: u32,) -> Self {
        ConditionPy {temporal_relationship, object, value, amount}
    }

    fn __repr__(&self) -> String {
        format!("Condition(temporal_relationship: {}, object: {}, value: (), amount)", self.temporal_relationship, self.object, self.value, self.amount)
    }
}

#[pymethods]
impl TokenPy {
    #[new]
    fn init(timeline_name: String, value: String, capacity: u32, const_time: TokenTimePy, conditions: Vec<ConditionPy>) -> Self {
        TokenPy {timeline_name, value, capacity, const_time, conditions}
    }

    fn __repr__(&self) -> String {
        format!("Token(timeline_name: {}, value: {}, capacity: {}, const_time: {}, conditions: {})", self.timeline_name, self.value, self.capacity, self.const_time, self.conditions)
    }
}

#[pymethods]
impl GroupPy {
    #[new]
    fn init(name: String, members: Vec<String>) -> Self {
        GroupPy { name, members }
    }

    fn __repr__(&self) -> String {
        format!("Group({}, {} members)", &self.name, self.members.len())
    }
}

#[pymethods]
impl SolutionPy {
    #[new]
    fn init(tokens: Vec<SolutionTokenPy>) -> Self {
        SolutionPy { tokens }
    }

    fn __repr__(&self) -> String {
        format!("Solution(tokens: {})", self.tokens)
    }
}

#[pymethods]
impl SolutionTokenPy {
    #[new]
    fn init(object_name: String, value: String, start_time: f32, end_time: f32) -> Self {
        SolutionTokenPy { object_name, value, start_time, end_time }
    }

    fn __repr__(&self) -> String {
        format!("SolutionToken(object_name: {}, value: {}, start_time: {}, end_time: {})", self.object_name, self.value, self.start_time, self.end_time)
    }
}

#[pyfunction]
fn solve(problem: ProblemPy) -> PyResult<SolutionPy> {
    let problem: Problem = problem_from_py(&problem);
    match solve(&problem, false) {
        Ok(s) => Ok(solution_to_py(&s)),
        Err(e) => Err(PyTypeErr::new("Solver returned error!")),
    }
}

#[pyfunction]
fn goal() -> TokenTimePy {
    None
}

#[pyfunction]
fn fact(a: Option<usize>, b: Option<usize>) -> TokenTimePy {
    Some((a,b))
}



/// A Python module implemented in Rust.
#[pymodule]
fn pyparaspace(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    m.add_function(wrap_pyfunction!(goal, m)?)?;
    m.add_function(wrap_pyfunction!(fact, m)?)?;

    m.add_class::<ProblemPy>()?;
    m.add_class::<TimelinePy>()?;
    m.add_class::<ValuePy>()?;
    m.add_class::<ConditionPy>()?;
    m.add_class::<TemporalRelationshipPy>()?;
    m.add_class::<GroupPy>()?;
    m.add_class::<TokenPy>()?;
    m.add_class::<SolutionPy>()?;
    m.add_class::<SolutionTokenPy>()?;

    Ok(())
}

// Utility for translating from pyparaspace to paraspace
fn problem_from_py(problem: &ProblemPy) -> Problem {
    let mut timelines: Vec<Timeline> = Vec::new();
    for p in problem.timelines.iter() {
        timelines.push(timeline_from_py(p));
    }

    let mut groups: Vec<Group> = Vec::new();
    for g in problem.groups.iter() {
        groups.push(group_from_py(g));
    }

    let mut tokens: Vec<Token> = Vec::new();
    for t in problem.tokens.iter() {
        tokens.push(token_from_py(t));
    }

    Problem {timelines, groups, tokens}
}

fn timeline_from_py(timeline: &TimelinePy) -> Timeline {
    let mut values: Vec<Value> = Vec::new();
    for v in timeline.values.iter() {
        values.push(value_from_py(&v));
    }
    Timeline {name: timeline.name, values}
}

fn value_from_py(value: &ValuePy) -> Value {
    let mut conditions: Vec<Condition> = Vec::new();
    for c in value.conditions.iter() {
        conditions.push(condition_from_py(&c));
    }
    Value {name: value.name, duration: value.duration, conditions, capacity: value.capacity}
}

fn condition_from_py(ConditionPy { temporal_relationship, object, amount, value }: &ConditionPy) -> Condition {
    let temporal_relationship = match temporal_relationship {
        TemporalRelationshipPy::MetBy => TemporalRelationship::MetBy,
        TemporalRelationshipPy::MetByTransitionFrom => TemporalRelationship::MetByTransitionFrom,
        TemporalRelationshipPy::Meets => TemporalRelationship::Meets,
        TemporalRelationshipPy::Cover => TemporalRelationship::Cover,
        TemporalRelationshipPy::Equal => TemporalRelationship::Equal,
        TemporalRelationshipPy::StartsAfter => TemporalRelationship::StartsAfter,
    };
    Condition { temporal_relationship, object, amount, value }
}

fn group_from_py(GroupPy {name, members}: &GroupPy) -> Group {
    Group {name, members}
}

fn token_from_py(token: &TokenPy) -> Token {

    let const_time = match token.const_time {
        Some(v) => TokenTime::Fact((v.0), (v.1)),
        None => TokenTime::Goal,
    };

    let mut conditions: Vec<Condition> = Vec::new();
    for c in value.conditions.iter() {
        conditions.push(condition_from_py(&c));
    }

    Token {timeline_name: token.timeline_name, value: token.value, capacity: token.capacity, const_time, conditions}
}

fn solution_to_py(solution: &Solution) -> SolutionPy {
    let mut tokens: Vec<SolutionTokenPy> = Vec::new();
    for t in solution.tokens.iter() {
        tokens.push(solutiontoken_to_py(&t));
    }
    SolutionPy {tokens}
}

fn solutiontoken_to_py(SolutionToken {object_name, value, start_time, end_time}: SolutionToken) -> SolutionTokenPy {
    SolutionTokenPy {object_name, value, start_time, end_time}
}