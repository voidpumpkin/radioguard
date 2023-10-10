use chrono::offset::Utc;
use chrono::DateTime;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::Type)]
pub struct Snapshot {
    pub id: i64,
    pub data_uri: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::Type)]
pub struct TagGroup {
    pub id: i64,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, sqlx::Type)]
pub struct Tag {
    pub id: i64,
    pub tag_group_id: i64,
    pub value: Option<String>,
}
