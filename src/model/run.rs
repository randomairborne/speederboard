use std::collections::HashMap;

use crate::id::{CategoryMarker, GameMarker, Id, RunMarker, UserMarker};

#[derive(serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, sqlx::Type)]
pub enum RunStatus {
    Verified,
    Rejected,
    Pending,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub enum RunMetadataEntry {
    ArbitraryString(String),
    Float(f64),
    Int(i64),
    Time(u64),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct Run {
    pub id: Id<RunMarker>,
    pub game: Id<GameMarker>,
    pub category: Id<CategoryMarker>,
    pub submitter: Id<UserMarker>,
    pub verifier: Option<Id<UserMarker>>,
    pub video: String,
    pub description: String,
    pub metadata: HashMap<String, RunMetadataEntry>,
}
