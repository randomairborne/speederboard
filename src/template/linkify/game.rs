use std::collections::HashMap;

use tera::Value;

use super::GetLinks;
use crate::model::Game;

#[derive(serde::Serialize)]
pub struct GameLinks {
    cover_art_url: String,
    banner_url: String,
    ui_url: String,
    edit_url: String,
    feed_url: String,
    team_url: String,
    forum_url: String,
    forum_new_post_url: String,
}

impl tera::Function for GetLinks<GameLinks> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let Some(value) = args.get("game") else {
            return Err(tera::Error::msg("getgamelinks() missing `game` argument"));
        };
        let ext = "webp";
        let game: Game = serde_json::from_value(value.clone())?;

        let cover_art_url = game.cover_art_url(&self.user_content, &self.root, ext);
        let banner_url = game.banner_url(&self.user_content, &self.root, ext);

        let ui_url = format!("{}/game/{}", self.root, game.slug);
        let edit_url = format!("{ui_url}/edit");
        let feed_url = format!("{ui_url}/feed");
        let team_url = format!("{ui_url}/team");
        let forum_url = format!("{}/forum/{}", self.root, game.slug);
        let forum_new_post_url = format!("{forum_url}/new");

        let links = GameLinks {
            cover_art_url,
            banner_url,
            ui_url,
            edit_url,
            feed_url,
            team_url,
            forum_url,
            forum_new_post_url,
        };
        Ok(serde_json::value::to_value(links)?)
    }

    fn is_safe(&self) -> bool {
        false
    }
}
