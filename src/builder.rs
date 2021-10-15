use std::rc::Rc;

use crate::problem::TimelinePlanningProblem;

pub struct TimelineProblemBuilder {
    resources: Vec<Resource>,
    timelines: Vec<TimelineBuilder>,
}

// Compilation idea:
//  Detect when two resources can be joined together into one
//  For example, in carbonara domain, boiling/cooking needs to select a plate,
//   and then use it exclusively, but if there are several plates they behave
//   just like if there was a resource with higher capacity. Symmetry reduction
//   effect by treating them as interchangable.
struct Resource {
    name: Rc<String>,
    class: Rc<String>,
    quantity: usize,
}

pub struct TimelineBuilder {
    states: Vec<State>,
}

impl TimelineBuilder {
    pub fn state(&mut self, name: &str) -> BuildState {
        let idx = self.states.len();
        BuildState {
            timeline_builder: self,
            state: State {
                min_duration: None,
                max_duration: None,
                resources: Vec::new(),
            },
        }
    }
}

pub struct BuildState<'a> {
    timeline_builder: &'a mut TimelineBuilder,
    state: State,
}

impl<'a> Drop for BuildState<'a> {
    fn drop(&mut self) {
        self.timeline_builder.states.push(self.state.clone());
    }
}

#[derive(Clone)]
pub struct State {
    min_duration: Option<usize>,
    max_duration: Option<usize>,
    resources: Vec<Rc<String>>,
}

impl<'a> BuildState<'a> {
    pub fn duration_eq(&mut self, x: usize) -> &mut Self {
        self.state.min_duration = Some(x);
        self.state.max_duration = Some(x);
        self
    }

    pub fn use_resource(&mut self, r: &str) -> &mut Self {
        self.state.resources.push(r.to_string().into());
        self
    }
}

impl TimelineProblemBuilder {
    pub fn new() -> Self {
        TimelineProblemBuilder {
            resources: Vec::new(),
            timelines: Vec::new(),
        }
    }

    pub fn reusable_resource(&mut self, class: &str, name: &str, quantity: usize) {
        self.resources.push(Resource {
            name: name.to_string().into(),
            class: class.to_string().into(),
            quantity,
        });
    }

    pub fn timeline(&mut self, name: &str) -> &mut TimelineBuilder {
        let idx = self.timelines.len();
        self.timelines.push(TimelineBuilder { states: Vec::new() });

        &mut self.timelines[idx]
    }

    pub fn build(self) -> TimelinePlanningProblem {
        todo!()
    }
}
