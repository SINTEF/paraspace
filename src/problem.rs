use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Problem {
    pub timelines: Vec<Timeline>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub value: String,
    pub capacity: u32,
    pub const_time: TokenTime,
    pub conditions: Vec<Vec<Condition>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TokenTime {
    Fact(Option<usize>, Option<usize>),
    Goal,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timeline {
    pub name: String,
    pub token_types: Vec<TokenType>,
    pub static_tokens: Vec<Token>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenType {
    pub value: String,
    pub duration_limits: (usize, Option<usize>),
    pub conditions: Vec<Vec<Condition>>,
    pub capacity: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Condition {
    pub timeline_ref: String,
    pub temporal_relationship: TemporalRelationship,
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
            && self.timeline_ref == timeline )
        .then(|| self.value.as_str())
    }
    pub fn is_timeline_transition_to(&self, timeline: &str) -> Option<&str> {
        (matches!(self.temporal_relationship, TemporalRelationship::Meets)
            && self.timeline_ref == timeline)
        .then(|| self.value.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TemporalRelationship {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Solution {
    pub timelines: Vec<SolutionTimeline>,
    pub end_of_time :f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolutionTimeline {
    pub name: String,
    pub tokens: Vec<SolutionToken>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolutionToken {
    pub value: String,
    pub start_time: f32,
    pub end_time: f32,
}
