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
    pub conditions :Vec<Condition>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TokenTime {
    Fact(Option<usize>, Option<usize>),
    Goal,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timeline {
    pub name: String,
    pub values: Vec<TokenType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenType {
    pub name: String,
    pub duration: (usize, Option<usize>),
    pub conditions: Vec<Condition>,
    pub capacity: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(Clone)]
pub struct Condition {
    pub temporal_relationship: TemporalRelationship,
    pub object: ObjectSet,
    pub value: String,
    pub amount: u32,
}

// impl Condition {
//     pub fn is_timeline_transition_from(&self, timeline: &str) -> Option<&str> {
//         (matches!(self.temporal_relationship, TemporalRelationship::MetBy)
//             && self.object == ObjectSet::Object(timeline.to_string()))
//         .then(|| self.value.as_str())
//     }
//     pub fn is_timeline_transition_to(&self, timeline: &str) -> Option<&str> {
//         (matches!(self.temporal_relationship, TemporalRelationship::Meets)
//             && self.object == ObjectSet::Object(timeline.to_string()))
//         .then(|| self.value.as_str())
//     }
// }

#[derive(Clone)]
#[derive(Serialize, Deserialize, Debug)]
pub enum TemporalRelationship {
    MetBy,
    MetByTransitionFrom,
    Meets,
    Cover,
    Equal,
}


#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectSet {
    Group(String),
    Object(String),
    Set(Vec<String>),
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
