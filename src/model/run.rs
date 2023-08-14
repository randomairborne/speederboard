use std::cmp::Ordering;

use crate::id::{CategoryMarker, GameMarker, Id, RunMarker, UserMarker};

use super::{Category, Game, User};

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy, sqlx::Type,
)]
pub enum RunStatus {
    Verified,
    Rejected,
    Pending,
}

impl From<i64> for RunStatus {
    fn from(value: i64) -> Self {
        match value.cmp(&0) {
            Ordering::Equal => Self::Pending,
            Ordering::Less => Self::Rejected,
            Ordering::Greater => Self::Verified,
        }
    }
}

impl From<i32> for RunStatus {
    fn from(value: i32) -> Self {
        i64::from(value).into()
    }
}

impl From<i16> for RunStatus {
    fn from(value: i16) -> Self {
        i64::from(value).into()
    }
}

impl From<i8> for RunStatus {
    fn from(value: i8) -> Self {
        i64::from(value).into()
    }
}

impl From<RunStatus> for i8 {
    fn from(value: RunStatus) -> Self {
        match value {
            RunStatus::Pending => 0,
            RunStatus::Rejected => -1,
            RunStatus::Verified => 1,
        }
    }
}

impl From<RunStatus> for i16 {
    fn from(value: RunStatus) -> Self {
        i16::from(i8::from(value))
    }
}

impl From<RunStatus> for i32 {
    fn from(value: RunStatus) -> Self {
        i32::from(i8::from(value))
    }
}

impl From<RunStatus> for i64 {
    fn from(value: RunStatus) -> Self {
        i64::from(i8::from(value))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct Run {
    pub id: Id<RunMarker>,
    pub game: Id<GameMarker>,
    pub category: Id<CategoryMarker>,
    pub submitter: Id<UserMarker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<Id<UserMarker>>,
    pub video: String,
    pub description: String,
    pub score: i64,
    pub time: i64,
    pub status: RunStatus,
}

#[derive(serde::Serialize, Debug, PartialEq, Clone)]
pub struct ResolvedRun<'a> {
    pub id: Id<RunMarker>,
    pub game: &'a Game,
    pub category: &'a Category,
    pub submitter: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<User>,
    pub video: String,
    pub description: String,
    pub score: i64,
    pub time: i64,
    pub status: RunStatus,
}
