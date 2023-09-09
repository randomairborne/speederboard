use axum::{
    extract::{Path, State},
    response::Redirect,
};

use crate::{
    id::{CategoryMarker, Id},
    model::{Category, Game, Permissions, User},
    template::BaseRenderInfo,
    util::{self, ValidatedForm},
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize, Debug, Clone)]
pub struct GameEditContext {
    game: Game,
    categories: Vec<Category>,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct GameEdit {
    #[garde(length(min = crate::util::MIN_GAME_NAME_LEN, max = crate::util::MAX_GAME_NAME_LEN))]
    name: String,
    #[garde(url, length(min = crate::util::MIN_GAME_URL_LEN, max = crate::util::MAX_GAME_URL_LEN))]
    url: String,
    #[garde(length(min = crate::util::MIN_GAME_DESCRIPTION_LEN, max = crate::util::MAX_GAME_DESCRIPTION_LEN))]
    description: String,
}

pub async fn get(
    State(state): State<AppState>,
    Path(game_slug): Path<String>,
    user: User,
    base: BaseRenderInfo,
) -> HandlerResult {
    let (game, member) = util::game_n_member(&state, user, &game_slug).await?;
    member.perms.check(Permissions::ADMINISTRATOR)?;
    let categories = query_as!(
        Category,
        "SELECT name, id, game, scoreboard,
        description, rules, flags
        FROM categories WHERE game = $1",
        game.id.get()
    )
    .fetch_all(&state.postgres)
    .await?;
    let context = GameEditContext {
        game,
        categories,
        base,
    };
    state.render("edit_game.jinja", context)
}

pub async fn edit(
    State(state): State<AppState>,
    Path(game_slug): Path<String>,
    user: User,
    ValidatedForm(form): ValidatedForm<GameEdit>,
) -> Result<Redirect, Error> {
    let (game, member) = util::game_n_member(&state, user, &game_slug).await?;
    member.perms.check(Permissions::ADMINISTRATOR)?;
    query!(
        "UPDATE games SET name = $1, url = $2, description = $3 WHERE id = $4",
        form.name,
        form.url,
        form.description,
        game.id.get()
    )
    .execute(&state.postgres)
    .await?;
    Ok(Redirect::to(&format!("/game/{}/edit", game.slug)))
}

pub async fn set_default_category(
    State(state): State<AppState>,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    user: User,
) -> Result<Redirect, Error> {
    let (game, member) = util::game_n_member(&state, user, &game_slug).await?;
    member.perms.check(Permissions::ADMINISTRATOR)?;
    query!(
        "UPDATE games SET default_category = $2 WHERE id = $1",
        game.id.get(),
        category_id.get()
    )
    .execute(&state.postgres)
    .await?;
    Ok(Redirect::to(&format!("/game/{game_slug}/edit")))
}
