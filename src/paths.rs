use crate::id::{GameMarker, Id, UserMarker};

pub fn user_stylesheet_path(id: Id<UserMarker>) -> String {
    format!("/users/{id}/style.css")
}

pub fn user_pfp_path(id: Id<UserMarker>, ext: &str) -> String {
    format!("/users/{id}/pfp.{ext}")
}

pub fn user_banner_path(id: Id<UserMarker>, ext: &str) -> String {
    format!("/users/{id}/banner.{ext}")
}

pub fn game_banner_path(id: Id<GameMarker>, ext: &str) -> String {
    format!("/games/{id}/banner.{ext}")
}

pub fn game_cover_art_path(id: Id<GameMarker>, ext: &str) -> String {
    format!("/games/{id}/cover_art.{ext}")
}
