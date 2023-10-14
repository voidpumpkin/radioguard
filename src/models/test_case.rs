use chrono::NaiveDateTime;

use super::tag::Tag;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestCase {
    pub id: i64,
    pub run_id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub tags: Vec<Tag>,
}
