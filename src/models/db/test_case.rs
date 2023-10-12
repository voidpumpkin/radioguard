use chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestCase {
    pub id: i64,
    pub run: i64,
    pub created_at: NaiveDateTime,
    pub tag_ids: Vec<i64>,
}
