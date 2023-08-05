use crate::user::FrontendUser;

#[derive(serde::Serialize)]
pub struct BaseRenderInfo<'a> {
    pub root_url: &'a str,
    pub cdn_root: &'a str,
    pub logged_in_user: Option<FrontendUser>,
}

impl<'a> BaseRenderInfo<'a> {
    pub fn new(root_url: &'a str, cdn_root: &'a str) -> Self {
        Self {
            root_url,
            cdn_root,
            logged_in_user: None,
        }
    }
}
