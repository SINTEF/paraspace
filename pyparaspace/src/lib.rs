use paraspace::problem::*;
use pyo3::prelude::*;

//
// PROBLEM
//

#[derive(FromPyObject, Debug)]
enum ProblemArgument {
    ProblemPy(ProblemPy),
    TimelinesList(Vec<TimelinePy>),
    TimelinesDict(indexmap::IndexMap<String, Vec<TimelineEntry>>),
}

#[derive(FromPyObject, Debug)]
enum TimelineEntry {
    TokenType(TokenTypePy),
    StaticToken(StaticTokenPy),
}

#[derive(Clone)]
#[pyclass(name = "Problem")]
#[derive(Debug)]
pub struct ProblemPy {
    #[pyo3(get)]
    pub timelines: Vec<TimelinePy>,
}

#[pymethods]
impl ProblemPy {
    #[new]
    fn init(timelines: Vec<TimelinePy>) -> Self {
        ProblemPy { timelines }
    }

    fn __repr__(&self) -> String {
        format!("Problem(n_timelines={})", self.timelines.len())
    }
}

//
// TIMELINE
//

#[pyclass(name = "Timeline")]
#[derive(Clone, Debug)]
pub struct TimelinePy {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub token_types: Vec<TokenTypePy>,
    #[pyo3(get)]
    pub static_tokens: Vec<StaticTokenPy>,
}

#[pymethods]
impl TimelinePy {
    #[new]
    fn init(
        name: String,
        token_types: Vec<TokenTypePy>,
        static_tokens: Vec<StaticTokenPy>,
    ) -> Self {
        TimelinePy {
            name,
            token_types,
            static_tokens,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Timeline(name: {}, token types: {}, static tokens: {}))",
            self.name,
            self.token_types
                .iter()
                .map(|v| v.__repr__())
                .collect::<Vec<_>>()
                .join(", "),
            self.static_tokens
                .iter()
                .map(|v| v.__repr__())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

//
// TOKEN TYPE
//

#[pyclass(name = "TokenType")]
#[derive(Clone, Debug)]
pub struct TokenTypePy {
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub duration_limits: (usize, Option<usize>),
    #[pyo3(get)]
    pub capacity: u32,
    pub conditions: Vec<Vec<TemporalCondPy>>,
}

#[pymethods]
impl TokenTypePy {
    #[new]
    fn init<'a>(
        value: String,
        duration_limits: (usize, Option<usize>),
        conditions: Vec<&'a PyAny>,
        capacity: u32,
    ) -> PyResult<Self> {
        Ok(TokenTypePy {
            value,
            duration_limits,
            conditions: convert_conditions_py(conditions)?,
            capacity,
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "TokenType(name: {}, duration: {:?}, conditions: {}, capacity: {})",
            self.value,
            self.duration_limits,
            repr_conditions(&self.conditions),
            self.capacity
        )
    }

    #[getter]
    fn conditions(&self) -> Vec<CondPy> {
        self.conditions
            .iter()
            .map(|c| {
                if c.len() == 1 {
                    CondPy::TemporalCond(c[0].clone())
                } else {
                    CondPy::OrCond(OrCondPy {
                        disjuncts: c.clone(),
                    })
                }
            })
            .collect()
    }
}

//
// STATIC TOKEN
//

type TokenTimePy = Option<(Option<usize>, Option<usize>)>;

#[pyclass(name = "StaticToken")]
#[derive(Clone, Debug)]

pub struct StaticTokenPy {
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub capacity: u32,
    #[pyo3(get)]
    pub const_time: TokenTimePy,
    pub conditions: Vec<Vec<TemporalCondPy>>,
}

#[pymethods]
impl StaticTokenPy {
    #[new]
    fn init<'a>(
        value: String,
        capacity: u32,
        const_time: TokenTimePy,
        conditions: Vec<&'a PyAny>,
    ) -> PyResult<Self> {
        Ok(StaticTokenPy {
            value,
            capacity,
            const_time,
            conditions: convert_conditions_py(conditions)?,
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "StaticToken(value: {}, capacity: {}, const_time: {:?}, conditions: {})",
            self.value,
            self.capacity,
            self.const_time,
            repr_conditions(&self.conditions),
        )
    }

    #[getter]
    fn conditions(&self) -> Vec<CondPy> {
        self.conditions
            .iter()
            .map(|c| {
                if c.len() == 1 {
                    CondPy::TemporalCond(c[0].clone())
                } else {
                    CondPy::OrCond(OrCondPy {
                        disjuncts: c.clone(),
                    })
                }
            })
            .collect()
    }
}

enum CondPy {
    OrCond(OrCondPy),
    TemporalCond(TemporalCondPy),
}

impl IntoPy<PyObject> for CondPy {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            CondPy::OrCond(o) => o.into_py(py),
            CondPy::TemporalCond(t) => t.into_py(py),
        }
    }
}

#[pyclass(name = "OrCond")]
#[derive(Clone, Debug)]
pub struct OrCondPy {
    #[pyo3(get)]
    pub disjuncts: Vec<TemporalCondPy>,
}

#[pymethods]
impl OrCondPy {
    #[new]
    fn init(disjuncts: Vec<TemporalCondPy>) -> Self {
        Self { disjuncts }
    }

    fn __repr__(&self) -> String {
        format!(
            "OrCond({})",
            self.disjuncts
                .iter()
                .map(|v| v.__repr__())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[pyclass(name = "TemporalCond")]
#[derive(Clone, Debug)]
pub struct TemporalCondPy {
    #[pyo3(get)]
    pub timeline: String,
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub temporal_relation: TemporalRelationPy,
    #[pyo3(get)]
    pub amount: u32,
}

#[pymethods]
impl TemporalCondPy {
    #[new]
    fn init(
        timeline: String,
        value: String,
        temporal_relation: TemporalRelationPy,
        amount: u32,
    ) -> Self {
        Self {
            timeline,
            value,
            temporal_relation,
            amount,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "TemporalCond(timeline={}, value={}, temporal_relation={:?}, amount={})",
            self.timeline, self.value, self.temporal_relation, self.amount
        )
    }
}

fn convert_conditions_py(conditions: Vec<&PyAny>) -> PyResult<Vec<Vec<TemporalCondPy>>> {
    let mut vec = Vec::new();
    for cond in conditions {
        if let Ok(or) = cond.extract::<OrCondPy>() {
            vec.push(or.disjuncts);
        } else if let Ok(cond) = cond.extract::<TemporalCondPy>() {
            vec.push(vec![cond]);
        } else {
            return Err(pyo3::exceptions::PyException::new_err(
                "Could not convert object to paraspace scondition.",
            ));
        }
    }
    Ok(vec)
}

fn repr_conditions(conds: &Vec<Vec<TemporalCondPy>>) -> String {
    conds
        .iter()
        .map(|v| {
            if v.len() == 1 {
                v[0].__repr__()
            } else {
                OrCondPy {
                    disjuncts: v.clone(),
                }
                .__repr__()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[pyclass(name = "TemporalRelation")]
#[derive(Clone, Debug)]
pub enum TemporalRelationPy {
    MetBy,
    MetByTransitionFrom,
    Meets,
    Starts,
    StartPrecond,
    StartEffect,
    Cover,
    Equal,
    StartsAfter,
}

//
// SOLUTION
//

#[pyclass(name = "Solution")]
#[derive(Clone)]
pub struct SolutionPy {
    #[pyo3(get)]
    pub timelines: Vec<SolutionTimelinePy>,
    #[pyo3(get)]
    pub end_of_time: f32,
}

#[pyclass(name = "SolutionTimeline")]
#[derive(Clone)]
pub struct SolutionTimelinePy {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub tokens: Vec<SolutionTokenPy>,
}

#[pyclass(name = "SolutionToken")]
#[derive(Clone)]
pub struct SolutionTokenPy {
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub start_time: f32,
    #[pyo3(get)]
    pub end_time: f32,
}

#[pymethods]
impl SolutionPy {
    fn __repr__(&self) -> String {
        format!(
            "Solution(end_of_time: {}, timelines: {})",
            self.end_of_time,
            self.timelines
                .iter()
                .map(|v| v.__repr__())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[pymethods]
impl SolutionTimelinePy {
    fn __repr__(&self) -> String {
        format!(
            "SolutionTimeline(name: {}, tokens: {})",
            self.name,
            self.tokens
                .iter()
                .map(|v| v.__repr__())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[pymethods]
impl SolutionTokenPy {
    fn __repr__(&self) -> String {
        format!(
            "SolutionToken(value: {}, start_time: {}, end_time: {})",
            self.value, self.start_time, self.end_time
        )
    }
}

#[pyfunction]
fn as_json(problem :ProblemArgument) -> PyResult<String> {
    let problem = convert_problem_arguments(problem);
    Ok(paraspace::to_json(&problem))
}

fn convert_problem_arguments(problem :ProblemArgument) -> Problem {
    match problem {
        ProblemArgument::ProblemPy(p) => convert_problem(p),
        ProblemArgument::TimelinesList(l) => Problem {
            timelines: l.into_iter().map(convert_timeline).collect(),
        },
        ProblemArgument::TimelinesDict(map) => Problem {
            timelines: map
                .into_iter()
                .map(|(name, entries)| {
                    let mut token_types = Vec::new();
                    let mut static_tokens = Vec::new();
                    for entry in entries {
                        match entry {
                            TimelineEntry::TokenType(tt) => {
                                token_types.push(convert_token_type(tt))
                            }
                            TimelineEntry::StaticToken(st) => {
                                static_tokens.push(convert_static_token(st))
                            }
                        }
                    }
                    Timeline {
                        name,
                        token_types,
                        static_tokens,
                    }
                })
                .collect(),
        },
    }
}

#[pyfunction]

fn solve(problem: ProblemArgument) -> PyResult<SolutionPy> {
    let problem = convert_problem_arguments(problem);
    match paraspace::transitionsolver::solve(&problem, &Default::default()) {
        Ok(s) => Ok(SolutionPy {
            timelines: s
                .timelines
                .into_iter()
                .map(|tl| SolutionTimelinePy {
                    name: tl.name,
                    tokens: tl
                        .tokens
                        .into_iter()
                        .map(|t| SolutionTokenPy {
                            value: t.value,
                            start_time: t.start_time,
                            end_time: t.end_time,
                        })
                        .collect(),
                })
                .collect(),
            end_of_time: s.end_of_time,
        }),
        Err(e) => Err(pyo3::exceptions::PyException::new_err(match e {
            paraspace::SolverError::NoSolution => "No solution found",
            paraspace::SolverError::GoalValueDurationLimit => "Goal value duration limit error",
            paraspace::SolverError::GoalStateMissing => "Goal state missing",
        })),
    }
}

#[pyfunction]
fn goal() -> TokenTimePy {
    None
}

#[pyfunction]
fn fact(a: Option<usize>, b: Option<usize>) -> TokenTimePy {
    Some((a, b))
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyparaspace(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    m.add_function(wrap_pyfunction!(as_json, m)?)?;
    m.add_function(wrap_pyfunction!(goal, m)?)?;
    m.add_function(wrap_pyfunction!(fact, m)?)?;

    m.add_class::<ProblemPy>()?;
    m.add_class::<TimelinePy>()?;
    m.add_class::<TokenTypePy>()?;
    m.add_class::<OrCondPy>()?;
    m.add_class::<TemporalCondPy>()?;
    m.add_class::<TemporalRelationPy>()?;
    m.add_class::<StaticTokenPy>()?;

    m.add_class::<SolutionPy>()?;
    m.add_class::<SolutionTokenPy>()?;

    Ok(())
}

fn convert_problem(problem: ProblemPy) -> Problem {
    Problem {
        timelines: problem
            .timelines
            .into_iter()
            .map(convert_timeline)
            .collect(),
    }
}

fn convert_timeline(timeline: TimelinePy) -> Timeline {
    Timeline {
        name: timeline.name.clone(),
        token_types: timeline
            .token_types
            .into_iter()
            .map(convert_token_type)
            .collect(),
        static_tokens: timeline
            .static_tokens
            .into_iter()
            .map(convert_static_token)
            .collect(),
    }
}

fn convert_token_type(tt: TokenTypePy) -> TokenType {
    TokenType {
        value: tt.value,
        capacity: tt.capacity,
        duration_limits: tt.duration_limits,
        conditions: convert_conditions(tt.conditions),
    }
}

fn convert_static_token(tt: StaticTokenPy) -> Token {
    Token {
        value: tt.value,
        capacity: tt.capacity,
        conditions: convert_conditions(tt.conditions),
        const_time: match tt.const_time {
            Some((a, b)) => TokenTime::Fact(a, b),
            None => TokenTime::Goal,
        },
    }
}

fn convert_conditions(cond: Vec<Vec<TemporalCondPy>>) -> Vec<Vec<Condition>> {
    cond.into_iter()
        .map(|cs| {
            cs.into_iter()
                .map(|c| Condition {
                    timeline_ref: c.timeline,
                    value: c.value,
                    amount: c.amount,
                    temporal_relationship: match c.temporal_relation {
                        TemporalRelationPy::MetBy => TemporalRelationship::MetBy,
                        TemporalRelationPy::MetByTransitionFrom => {
                            TemporalRelationship::MetByTransitionFrom
                        }
                        TemporalRelationPy::Meets => TemporalRelationship::Meets,
                        TemporalRelationPy::Cover => TemporalRelationship::Cover,
                        TemporalRelationPy::Equal => TemporalRelationship::Equal,
                        TemporalRelationPy::StartsAfter => TemporalRelationship::StartsAfter,
                        TemporalRelationPy::Starts => TemporalRelationship::Starts,
                        TemporalRelationPy::StartPrecond => TemporalRelationship::StartPrecond,
                        TemporalRelationPy::StartEffect => TemporalRelationship::StartEffect,
                    },
                })
                .collect()
        })
        .collect()
}

