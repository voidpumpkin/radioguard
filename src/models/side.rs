use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;

#[derive(
    Debug, Clone, Copy, EnumString, Serialize, Deserialize, strum::Display, PartialEq, Eq, Hash,
)]
#[strum(serialize_all = "snake_case")]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}
