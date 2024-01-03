use chrono::DateTime;
use chrono::Utc;

use super::step::Step;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestCase {
    pub id: i64,
    pub run_id: i64,
    pub name: String,
    pub ignore_ranges: Vec<((u32, u32), (u32, u32))>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestCaseWithSteps {
    pub id: i64,
    pub run_id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub steps: Vec<Step>,
}
