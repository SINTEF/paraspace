use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Problem {
    pub timelines: Vec<Timeline>,
    pub groups: Vec<Group>,
    pub tokens: Vec<Token>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub timeline_name: String,
    pub value: String,
    pub capacity: u32,
    pub const_time: TokenTime,
    pub conditions: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TokenTime {
    Fact(Option<usize>, Option<usize>),
    Goal,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timeline {
    pub name: String,
    pub values: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Value {
    pub name: String,
    pub duration: (usize, Option<usize>),
    pub conditions: Vec<Condition>,
    pub capacity: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Condition {
    pub temporal_relationship: TemporalRelationship,
    pub object: ObjectSet,
    pub value: String,
    pub amount: u32,
}

impl Condition {
    pub fn is_timeline_transition_from(&self, timeline: &str) -> Option<&str> {
        ((matches!(self.temporal_relationship, TemporalRelationship::MetBy)
            || matches!(
                self.temporal_relationship,
                TemporalRelationship::MetByTransitionFrom
            ))
            && self.object.is_singleton_of(timeline))
        .then(|| self.value.as_str())
    }
    pub fn is_timeline_transition_to(&self, timeline: &str) -> Option<&str> {
        (matches!(self.temporal_relationship, TemporalRelationship::Meets)
            && self.object.is_singleton_of(timeline))
        .then(|| self.value.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TemporalRelationship {
    MetBy,
    MetByTransitionFrom,
    Meets,
    Cover,
    Equal,
    StartsAfter,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectSet {
    Group(String),
    Set(Vec<String>),
    Object(String),
}

impl ObjectSet {
    pub fn is_singleton_of(&self, name: &str) -> bool {
        match self {
            ObjectSet::Group(_g) => false,
            ObjectSet::Set(x) => x.len() == 1 && x[0] == name,
            ObjectSet::Object(o) => o == name,
        }
    }
}

//
// SOLUTION
//

#[derive(Serialize, Deserialize, Debug)]
pub struct Solution {
    pub tokens: Vec<SolutionToken>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolutionToken {
    pub object_name: String,
    pub value: String,
    pub start_time: f32,
    pub end_time: f32,
}
