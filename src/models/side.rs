use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;

#[derive(Debug, Clone, Copy, EnumString, Serialize, Deserialize, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Side {
    Left,
    Right,
}
