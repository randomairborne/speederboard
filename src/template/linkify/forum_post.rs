use std::collections::HashMap;

use tera::Value;

use super::GetLinks;
use crate::model::{ForumPost, Game};

#[derive(serde::Serialize)]
pub struct ForumPostLinks {
    ui_url: String,
}

impl tera::Function for GetLinks<ForumPostLinks> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let Some(post_val) = args.get("post") else {
            return Err(tera::Error::msg("getpostlinks() missing `post` argument"));
        };
        let Some(game_val) = args.get("game") else {
            return Err(tera::Error::msg("getpostlinks() missing `game` argument"));
        };

        let post: ForumPost = serde_json::from_value(post_val.clone())?;
        let game: Game = serde_json::from_value(game_val.clone())?;

        let ui_url = format!(
            "{}/forum/{}/post/{}",
            self.state.config.root_url, game.slug, post.id
        );

        let links = ForumPostLinks { ui_url };
        Ok(serde_json::value::to_value(links)?)
    }

    fn is_safe(&self) -> bool {
        false
    }
}
