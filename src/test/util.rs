use std::sync::atomic::AtomicUsize;

use chrono::NaiveDateTime;

use crate::{
    id::Id,
    model::{Category, Game, User},
};

pub static REDIS_DB_NUM: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn test_user() -> User {
    User {
        id: Id::new(1),
        username: "test".to_string(),
        stylesheet: false,
        biography: "".to_string(),
        pfp: false,
        banner: false,
        admin: false,
        created_at: NaiveDateTime::UNIX_EPOCH,
        flags: 0,
        language: None,
    }
}
pub(crate) fn test_category() -> Category {
    Category {
        id: Id::new(1),
        game: Id::new(1),
        name: "test category".to_string(),
        description: "test category".to_string(),
        rules: "(test)".to_string(),
        scoreboard: false,
        flags: 0,
    }
}
pub(crate) fn test_game() -> Game {
    Game {
        id: Id::new(1),
        name: "Test game".to_string(),
        slug: "test".to_string(),
        url: "https://example.com".to_string(),
        default_category: Id::new(1),
        description: "Test game for speederboard".to_string(),
        banner: false,
        cover_art: false,
        flags: 0,
    }
}
