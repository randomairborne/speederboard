use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};

use crate::{
    model::{Category, Game, Member, Permissions, User},
    template::BaseRenderInfo,
    util::ValidatedForm,
    AppState, Error,
};

#[derive(serde::Serialize)]
pub struct GameEditContext {
    game: Game,
    categories: Vec<Category>,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

#[derive(serde::Deserialize, garde::Validate)]
pub struct GameEdit {
    #[garde(length(min = crate::util::MIN_GAME_NAME_LEN, max = crate::util::MAX_GAME_NAME_LEN))]
    name: String,
    #[garde(length(min = crate::util::MIN_GAME_URL_LEN, max = crate::util::MAX_GAME_URL_LEN))]
    url: String,
    #[garde(length(min = crate::util::MIN_GAME_DESCRIPTION_LEN, max = crate::util::MAX_GAME_DESCRIPTION_LEN))]
    description: String,
}

pub async fn get(
    State(state): State<AppState>,
    Path(game_slug): Path<String>,
    user: User,
    base: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let member = Member::from_db(&state, user.id, game.id)
        .await?
        .ok_or(Error::NotFound)?;
    if !member.perms.contains(Permissions::ADMINISTRATOR) && !member.user.admin {
        return Err(Error::InsufficientPermissions);
    }
    let categories = query_as!(
        Category,
        "SELECT name, id, game, scoreboard, description,
            rules FROM categories WHERE game = $1",
        game.id.get()
    )
    .fetch_all(&state.postgres)
    .await?;
    let context = GameEditContext {
        game,
        categories,
        base,
    };
    let ctx = tera::Context::from_serialize(context)?;
    Ok(Html(state.tera.render("edit_game.jinja", &ctx)?))
}

pub async fn edit(
    State(state): State<AppState>,
    Path(game_slug): Path<String>,
    user: User,
    ValidatedForm(form): ValidatedForm<GameEdit>,
) -> Result<Redirect, Error> {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let member = Member::from_db(&state, user.id, game.id)
        .await?
        .ok_or(Error::InsufficientPermissions)?;
    if !member.perms.contains(Permissions::ADMINISTRATOR) && !member.user.admin {
        return Err(Error::InsufficientPermissions);
    }
    Ok(Redirect::to(&format!("/game/{}/edit", game.slug)))
}

pub async fn set_default_category() {}
