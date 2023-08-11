use crate::id::{CategoryMarker, GameMarker, Id};

#[derive(serde::Serialize, serde::Deserialize, Debug, Encode, Hash, PartialEq, Eq, Clone)]
pub struct Category {
    pub id: Id<CategoryMarker>,
    pub game: Id<GameMarker>,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub rules: String,
    pub sortby_field: String,
    pub sort_ascending: bool,
}

