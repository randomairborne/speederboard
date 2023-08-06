use crate::user::User;

#[derive(serde::Serialize)]
pub struct BaseRenderInfo<'a> {
    pub root_url: &'a str,
    pub cdn_url: &'a str,
    pub logged_in_user: Option<User>,
}

impl<'a> BaseRenderInfo<'a> {
    pub fn new(root_url: &'a str, cdn_url: &'a str) -> Self {
        Self {
            root_url,
            cdn_url,
            logged_in_user: None,
        }
    }
}
