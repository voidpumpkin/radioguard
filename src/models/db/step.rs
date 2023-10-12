use chrono::NaiveDateTime;

use super::tag::Tag;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Step {
    pub id: i64,
    pub data_uri: String,
    pub created_at: NaiveDateTime,
    pub parent_test_id: Option<i64>,
    pub parent_step_id: Option<i64>,
    pub tags: Vec<Tag>,
}
