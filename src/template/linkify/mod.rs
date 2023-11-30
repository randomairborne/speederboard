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

use crate::AppState;

pub struct GetLinks<T> {
    state: AppState,
    kind: PhantomData<T>,
}

impl<T> GetLinks<T> {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            kind: PhantomData,
        }
    }
}
