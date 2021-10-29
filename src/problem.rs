use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Problem {
    pub timelines :Vec<Timeline>,
    pub groups: Vec<Group>,
    pub tokens :Vec<Token>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Group {
    pub name :String,
    pub members :Vec<String>,
}


#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Token {
    pub timeline_name :String,
    pub value :String,
    pub capacity :u32,
    pub const_time :TokenTime,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum TokenTime {
    Fact(Option<usize>, Option<usize>),
    Goal,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Timeline {
    pub name :String,
    pub values :Vec<Value>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Value {
    pub name :String,
    pub duration :(usize,Option<usize>),
    pub conditions :Vec<Condition>,
    pub capacity :u32,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Condition {
    pub temporal_relationship: TemporalRelationship,
    pub object: ObjectSet,
    pub value: String,
    pub amount: u32,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum TemporalRelationship {
    Meet,
    Cover,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectSet {
    Group(String),
    Object(String),
}

//
// SOLUTION
//


#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Solution {
    pub tokens: Vec<SolutionToken>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct SolutionToken {
    pub object_name :String,
    pub value :String,
    pub start_time :f32,
    pub end_time :f32,
}

