use std::fmt::Display;

use chrono::DateTime;
use chrono::Utc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Step {
    pub id: i64,
    pub name: String,
    pub data_uri: String,
    pub created_at: DateTime<Utc>,
    pub test_case_id: i64,
    pub children_steps: Vec<Step>,
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.name)?;

        for child in &self.children_steps {
            writeln!(f, "    {child}")?;
        }

        Ok(())
    }
}
