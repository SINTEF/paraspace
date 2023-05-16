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
    pub conditions :Vec<ConditionPy>,
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
        format!("Timeline(name: {}, values: {})", self.name, self.values.len())
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

#[pyfunction]
fn solve(s: String) -> String {
    paraspace::solve_json(s)
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

    Ok(())
}