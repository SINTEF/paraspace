#![allow(unused)]
use std::rc::Rc;

pub struct TimelinePlanningProblem {
    pub timelines: Vec<Timeline>,
    pub actions: Vec<Action>,

    pub facts: Vec<()>,
    pub goals: Vec<()>,
}

pub struct Timeline {
    pub     values :Vec<Rc<String>>,
    
    // for each value, which values can it transition to (indices into `values`)
    pub transitions: Vec<Vec<usize>>, 

    // TODO controllability?
    // TODO duration?
}

pub struct TimelineValue {

}

pub struct Action {

}
