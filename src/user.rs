use crate::id::{Id, UserMarker};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AuthUser {
    pub id: Id<UserMarker>,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FrontendUser {
    pub id: Id<UserMarker>,
    pub username: String,
}
