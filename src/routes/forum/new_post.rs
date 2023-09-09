use axum::{
    extract::{Path, State},
    response::Redirect,
};

use crate::{
    model::{Game, User},
    template::BaseRenderInfo,
    util::ValidatedForm,
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize, Debug, Clone)]
pub struct PostCreatePage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    user: User,
    game: Game,
}

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct PostCreateForm {
    #[garde(length(max = crate::util::MAX_FORUM_TITLE_LEN, min = crate::util::MIN_FORUM_TITLE_LEN))]
    title: String,
    #[garde(length(max = crate::util::MAX_FORUM_POST_LEN, min = crate::util::MIN_FORUM_POST_LEN))]
    content: String,
}

pub async fn get(
    State(state): State<AppState>,
    user: User,
    base: BaseRenderInfo,
    Path(game_slug): Path<String>,
) -> HandlerResult {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let context = PostCreatePage { base, user, game };
    state.render("new_post.jinja", context)
}

#[allow(clippy::unused_async)]
pub async fn post(
    State(state): State<AppState>,
    user: User,
    Path(game_slug): Path<String>,
    ValidatedForm(form): ValidatedForm<PostCreateForm>,
) -> Result<Redirect, Error> {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let post_id = query!(
        "INSERT INTO forum_entries
        (
            title, game, author, content,
            created_at, flags
        )
        VALUES ($1, $2, $3, $4, NOW(), 0)
        RETURNING id",
        form.title,
        game.id.get(),
        user.id.get(),
        form.content
    )
    .fetch_one(&state.postgres)
    .await?
    .id;
    Ok(Redirect::to(&format!(
        "{}/forum/{game_slug}/post/{post_id}",
        state.config.root_url
    )))
}
