use chrono::DateTime;
use chrono::Utc;

use super::tag::Tag;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Run {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
}
