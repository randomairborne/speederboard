use std::collections::HashMap;

use tera::Value;

use super::GetLinks;
use crate::model::{Game, ResolvedRun};

#[derive(serde::Serialize)]
pub struct RunLinks {
    review_url: String,
    verify_post_url: String,
    reject_post_url: String,
    ui_url: String,
}

impl tera::Function for GetLinks<RunLinks> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let Some(game_val) = args.get("game") else {
            return Err(tera::Error::msg("getrunlinks() missing `game` argument"));
        };
        let Some(run_val) = args.get("run") else {
            return Err(tera::Error::msg("getrunlinks() missing `run` argument"));
        };

        let game: Game = serde_json::from_value(game_val.clone())?;
        let run: ResolvedRun = serde_json::from_value(run_val.clone())?;

        let ui_url = format!(
            "{}/game/{}/category/{}/run/{}",
            self.root, game.slug, run.category.id, run.id
        );
        let review_url = format!("{ui_url}/review");
        let verify_post_url = format!("{ui_url}/verify");
        let reject_post_url = format!("{ui_url}/reject");

        let links = RunLinks {
            ui_url,
            review_url,
            verify_post_url,
            reject_post_url,
        };
        Ok(serde_json::value::to_value(links)?)
    }

    fn is_safe(&self) -> bool {
        false
    }
}
