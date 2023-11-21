use super::User;
use crate::id::{ForumEntryMarker, Id};

#[derive(serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ForumPost {
    pub id: Id<ForumEntryMarker>,
    pub title: String,
    pub author: User,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub edited_at: Option<chrono::NaiveDateTime>,
    pub flags: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ForumComment {
    pub id: Id<ForumEntryMarker>,
    pub parent: Id<ForumEntryMarker>,
    pub author: User,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub edited_at: Option<chrono::NaiveDateTime>,
    pub flags: i64,
}
