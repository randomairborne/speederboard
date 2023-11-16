use std::{collections::HashMap, marker::PhantomData};

use tera::Value;

use super::GetLinks;
use crate::{config::Config, model::Game};

#[derive(serde::Serialize)]
pub struct GameLinks {
    cover_art_url: String,
    banner_url: String,
    ui_url: String,
}

impl GetLinks<GameLinks> {
    pub fn new(config: &Config) -> Self {
        Self {
            root: config.root_url.clone(),
            static_content: config.static_url.clone(),
            user_content: config.user_content_url.clone(),
            kind: PhantomData,
        }
    }
}

impl tera::Function for GetLinks<GameLinks> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let Some(value) = args.get("game") else {
            return Err(tera::Error::msg("getgamelinks() missing `game` argument"));
        };
        let ext = "webp";
        let game: Game = serde_json::from_value(value.clone())?;

        let cover_art_url = game.cover_art_url(&self.user_content, &self.static_content, ext);
        let banner_url = game.banner_url(&self.user_content, &self.static_content, ext);
        let ui_url = format!("{}/game/{}", self.root, game.slug);

        let links = GameLinks {
            cover_art_url,
            banner_url,
            ui_url,
        };
        Ok(serde_json::value::to_value(links)?)
    }

    fn is_safe(&self) -> bool {
        false
    }
}
