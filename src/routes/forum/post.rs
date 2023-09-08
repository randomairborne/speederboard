use axum::extract::State;

use crate::{
    model::{ForumComment, Game, ForumPost},
    template::BaseRenderInfo,
    AppState, HandlerResult,
};

#[derive(serde::Serialize, Clone, Debug)]
pub struct ForumPostPage {
    #[serde(flatten)]
    core: BaseRenderInfo,
    comments: Vec<ForumComment>,
    post: ForumPost,
    game: Game,
}

pub async fn get(State(state): State<AppState>, core: BaseRenderInfo) -> HandlerResult {
    let page = ForumPostPage { core };
    state.render("forum_post.jinja", ())
}
