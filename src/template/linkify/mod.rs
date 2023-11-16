// TODO: Linkify other resources

use std::marker::PhantomData;

pub struct GetLinks<T> {
    root: String,
    static_content: String,
    user_content: String,
    kind: PhantomData<T>,
}

pub mod category;
pub mod game;
pub mod run;
pub mod user;
