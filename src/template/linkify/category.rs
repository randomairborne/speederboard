use std::collections::HashMap;

use tera::Value;

use super::GetLinks;
use crate::model::{Game, MiniCategory};

#[derive(serde::Serialize)]
pub struct CategoryLinks {
    ui_url: String,
    feed_url: String,
    edit_url: String,
    new_run_url: String,
}

impl tera::Function for GetLinks<CategoryLinks> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let Some(category_val) = args.get("category") else {
            return Err(tera::Error::msg(
                "getcategorylinks() missing `category` argument",
            ));
        };
        let Some(game_val) = args.get("game") else {
            return Err(tera::Error::msg(
                "getcategorylinks() missing `game` argument",
            ));
        };

        let game: Game = serde_json::from_value(game_val.clone())?;
        let category: MiniCategory = serde_json::from_value(category_val.clone())?;

        if category.game != game.id {
            return Err(tera::Error::msg(
                "property `game` of `category` and property `id` of `game` do not match",
            ));
        }

        let ui_url = format!("{}/game/{}/category/{}", self.root, game.slug, category.id);
        let feed_url = format!("{ui_url}/feed");
        let edit_url = format!("{ui_url}/edit");
        let new_run_url = format!("{ui_url}/run/new");

        let links = CategoryLinks {
            ui_url,
            feed_url,
            edit_url,
            new_run_url,
        };
        Ok(serde_json::value::to_value(links)?)
    }

    fn is_safe(&self) -> bool {
        false
    }
}
