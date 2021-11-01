use std::collections::HashSet;

use crate::problem::Problem;

/// Check if a timelines' value can be reached multiple times or not.
pub fn multiplicity_one(_problem :&Problem) -> HashSet<(&str,&str)> {
    HashSet::new()    
}