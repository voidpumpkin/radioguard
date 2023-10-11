use chrono::offset::Utc;
use chrono::DateTime;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Run {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestCase {
    pub id: i64,
    pub run: i64,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Step {
    pub id: i64,
    pub data_uri: String,
    pub created_at: DateTime<Utc>,
    pub parent_test_id: Option<i64>,
    pub parent_step_id: Option<i64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag {
    pub id: i64,
    pub value: String,
}
