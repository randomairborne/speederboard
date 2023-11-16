use std::{collections::HashMap, marker::PhantomData};

use tera::Value;

use super::GetLinks;
use crate::{config::Config, model::User};

#[derive(serde::Serialize)]
pub struct UserLinks {
    pfp_url: String,
    banner_url: String,
    stylesheet_url: String,
    ui_url: String,
}

impl GetLinks<UserLinks> {
    pub fn new(config: &Config) -> Self {
        Self {
            root: config.root_url.clone(),
            static_content: config.static_url.clone(),
            user_content: config.user_content_url.clone(),
            kind: PhantomData,
        }
    }
}

impl tera::Function for GetLinks<UserLinks> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let Some(value) = args.get("user") else {
            return Err(tera::Error::msg("getuserlinks() missing `user` argument"));
        };
        let ext = "webp";
        let user: User = serde_json::from_value(value.clone())?;

        let pfp_url = user.pfp_url(&self.user_content, &self.static_content, ext);
        let banner_url = user.banner_url(&self.user_content, &self.static_content, ext);
        let stylesheet_url = user.stylesheet_url(&self.user_content);
        let ui_url = format!("{}/user/{}", self.root, user.username);

        let links = UserLinks {
            pfp_url,
            banner_url,
            stylesheet_url,
            ui_url,
        };

        Ok(serde_json::value::to_value(links)?)
    }

    fn is_safe(&self) -> bool {
        false
    }
}
