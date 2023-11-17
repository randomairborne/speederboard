// TODO: Linkify other resources
mod category;
mod forum_post;
mod game;
mod run;
mod user;

use std::marker::PhantomData;

pub use category::CategoryLinks;
pub use forum_post::ForumPostLinks;
pub use game::GameLinks;
pub use run::RunLinks;
pub use user::UserLinks;

use crate::config::Config;

pub struct GetLinks<T> {
    root: String,
    static_content: String,
    user_content: String,
    kind: PhantomData<T>,
}

impl<T> GetLinks<T> {
    pub fn new(config: &Config) -> Self {
        Self {
            root: config.root_url.clone(),
            static_content: config.static_url.clone(),
            user_content: config.user_content_url.clone(),
            kind: PhantomData,
        }
    }
}
