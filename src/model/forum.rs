use crate::id::{Id, ForumEntryMarker, UserMarker};

#[derive(serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ForumPost {
    pub id: Id<ForumEntryMarker>,
    pub title: String,
    pub author: Id<UserMarker>,
    pub content: String,
    pub flags: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ForumComment {
    pub id: Id<ForumEntryMarker>,
    pub parent: Id<ForumEntryMarker>,
    pub root: Id<ForumEntryMarker>,
    pub author: Id<UserMarker>,
    pub content: String,
    pub flags: i64,
}