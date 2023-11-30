use std::collections::HashMap;

use tera::Value;

use super::GetLinks;
use crate::model::User;

#[derive(serde::Serialize)]
pub struct UserLinks {
    pfp_url: String,
    banner_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stylesheet_url: Option<String>,
    ui_url: String,
}

impl tera::Function for GetLinks<UserLinks> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let Some(value) = args.get("user") else {
            return Err(tera::Error::msg("getuserlinks() missing `user` argument"));
        };
        let ext = "webp";
        let user: User = serde_json::from_value(value.clone())?;

        let pfp_url = user.pfp_url(&self.state, ext);
        let banner_url = user.banner_url(&self.state, ext);
        let stylesheet_url = user.stylesheet_url(&self.state);
        let ui_url = format!("{}/user/{}", self.state.config.root_url, user.username);

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
