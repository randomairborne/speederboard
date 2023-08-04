#[derive(serde::Serialize)]
pub struct BaseRenderInfo<'a> {
    pub root_url: &'a str,
}

impl<'a> BaseRenderInfo<'a> {
    pub fn new(root_url: &'a str) -> Self {
        Self { root_url }
    }
}
