use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Problem {
    pub resources :Vec<Resource>,
    pub timelines :Vec<Timeline>,

    /// A specific timeline has a specific value.
    pub facts :Vec<TimelineValue>,

    /// A specific timeline has a specific value.
    pub goals :Vec<TimelineValue>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct TimelineValue {
    pub timeline_name :String,
    pub value :String,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Resource {
    pub class :String,
    pub name :Option<String>,
    pub capacity :usize,
}


#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Timeline {
    pub class :String,
    pub name :Option<String>,
    pub states :Vec<State>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct State {
    pub name :String,
    pub duration :(usize,Option<usize>),
    pub conditions :Vec<Condition>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Condition {
    UseResource(ObjectRef, usize),
    TransitionFrom(String),
    During(ObjectRef, String),
    MetBy(ObjectRef, String),
}

/// Refers to an object, i.e. a timeline or a resource:
/// either any object of a given class or a specific object by name.
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum ObjectRef {
    AnyOfClass(String),
    Named(String),
}