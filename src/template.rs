use crate::user::FrontendUser;

#[derive(serde::Serialize)]
pub struct BaseRenderInfo<'a> {
    pub root_url: &'a str,
    pub logged_in_user: Option<FrontendUser>
}

impl<'a> BaseRenderInfo<'a> {
    pub fn new(root_url: &'a str) -> Self {
        Self { root_url, logged_in_user: None }
    }
}
