use pyo3::prelude::*;

#[pyclass(name = "Problem")]
pub struct ProblemPy {
    #[pyo3(get)]
    pub groups: Vec<GroupPy>,
}

#[pyclass(name = "Group")]
#[derive(Clone)]
pub struct GroupPy {
    pub name: String,
    pub members: Vec<String>,
}

#[pymethods]
impl ProblemPy {
    #[new]
    fn init(group: Vec<GroupPy>) -> Self {
        ProblemPy { groups: group }
    }

    fn __repr__(&self) -> String {
        format!("Problem({} groups)", self.groups.len())
    }
}

#[pymethods]
impl GroupPy {
    #[new]
    fn init(name: String, members: Vec<String>) -> Self {
        GroupPy { name, members }
    }

    fn add_member(&mut self, name: String) {
        self.members.push(name);
    }

    fn __repr__(&self) -> String {
        format!("Group({}, {} members)", &self.name, self.members.len())
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn solve(s: String) -> String {
    timelinemodel::solve_json(s)
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyparaspace(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(solve, m)?)?;

    m.add_class::<ProblemPy>()?;
    m.add_class::<GroupPy>()?;

    Ok(())
}
