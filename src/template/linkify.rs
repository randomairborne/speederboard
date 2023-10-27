use std::collections::HashMap;

use tera::Value;

use crate::{
    config::Config,
    model::{Game, User},
};

pub struct GetUserLinks {
    root: String,
    static_content: String,
    user_content: String,
}

impl GetUserLinks {
    pub fn new(config: &Config) -> Self {
        Self {
            root: config.root_url.clone(),
            static_content: config.static_url.clone(),
            user_content: config.user_content_url.clone(),
        }
    }
}

impl tera::Function for GetUserLinks {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let mut json_map = serde_json::Map::with_capacity(8);
        let Some(value) = args.get("user") else {
            return Err(tera::Error::msg("getuserlinks() missing `user` argument"));
        };
        let ext = "png";
        let user: User = serde_json::from_value(value.clone())?;
        json_map.insert(
            "pfp_url".to_owned(),
            Value::String(user.pfp_url(&self.user_content, &self.static_content, ext)),
        );
        json_map.insert(
            "banner_url".to_owned(),
            Value::String(user.banner_url(&self.user_content, ext)),
        );
        json_map.insert(
            "stylesheet_url".to_owned(),
            Value::String(user.stylesheet_url(&self.user_content)),
        );
        json_map.insert(
            "ui_url".to_owned(),
            Value::String(format!("{}/user/{}", self.root, user.username)),
        );

        Ok(Value::Object(json_map))
    }

    fn is_safe(&self) -> bool {
        false
    }
}

pub struct GetGameLinks {
    root: String,
    static_content: String,
    user_content: String,
}

impl GetGameLinks {
    pub fn new(config: &Config) -> Self {
        Self {
            root: config.root_url.clone(),
            static_content: config.static_url.clone(),
            user_content: config.user_content_url.clone(),
        }
    }
}

impl tera::Function for GetGameLinks {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let mut json_map = serde_json::Map::with_capacity(8);
        let Some(value) = args.get("game") else {
            return Err(tera::Error::msg("getgamelinks() missing `game` argument"));
        };
        let ext = "png";
        let game: Game = serde_json::from_value(value.clone())?;

        let cover_art = if game.cover_art {
            game.cover_art_url(&self.user_content, &self.static_content, ext)
        } else {
            format!("{}/defaults/coverart.svg", self.static_content)
        };
        json_map.insert("cover_art_url".to_owned(), Value::String(cover_art));

        let banner = if game.cover_art {
            game.banner_url(&self.user_content, ext)
        } else {
            format!("{}/defaults/banner.svg", self.static_content)
        };

        json_map.insert("banner_url".to_owned(), Value::String(banner));

        json_map.insert(
            "ui_url".to_owned(),
            Value::String(format!("{}/game/{}", self.root, game.slug)),
        );

        Ok(Value::Object(json_map))
    }

    fn is_safe(&self) -> bool {
        false
    }
}
