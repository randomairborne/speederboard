use axum::{extract::State, response::Redirect};

use crate::{
    model::User, template::BaseRenderInfo, util::ValidatedForm, AppState, Error, HandlerResult,
};

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct GameCreateForm {
    #[garde(length(min = crate::util::MIN_GAME_NAME_LEN, max = crate::util::MAX_GAME_NAME_LEN))]
    name: String,
    #[garde(length(min = crate::util::MIN_GAME_SLUG_LEN, max = crate::util::MAX_GAME_SLUG_LEN), custom(crate::util::validate_slug))]
    slug: String,
    #[garde(url, length(min = crate::util::MIN_GAME_URL_LEN, max = crate::util::MAX_GAME_URL_LEN))]
    url: String,
    #[garde(length(min = crate::util::MIN_GAME_DESCRIPTION_LEN, max = crate::util::MAX_GAME_DESCRIPTION_LEN))]
    description: String,
    #[garde(length(min = crate::util::MIN_CATEGORY_NAME_LEN, max = crate::util::MAX_CATEGORY_NAME_LEN))]
    cat_name: String,
    #[garde(length(min = crate::util::MIN_CATEGORY_DESCRIPTION_LEN, max = crate::util::MAX_CATEGORY_DESCRIPTION_LEN))]
    cat_description: String,
    #[garde(length(min = crate::util::MIN_CATEGORY_RULES_LEN, max = crate::util::MAX_CATEGORY_RULES_LEN))]
    cat_rules: String,
    #[garde(skip)]
    #[serde(default = "crate::util::return_false")]
    scoreboard: bool,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct GameCreatePage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    user: User,
}

#[allow(clippy::unused_async)]
pub async fn get(State(state): State<AppState>, user: User, base: BaseRenderInfo) -> HandlerResult {
    let ctx = GameCreatePage { base, user };
    state.render("create_game.jinja", ctx)
}

pub async fn post(
    State(state): State<AppState>,
    ValidatedForm(newgame): ValidatedForm<GameCreateForm>,
) -> Result<Redirect, Error> {
    let mut trans = state.postgres.begin().await?;
    let game_id = query!(
        "INSERT INTO games
        (
            name, slug, url, description,
            has_stylesheet, default_category
        )
        VALUES ($1, $2, $3, $4, false, -1) RETURNING id",
        newgame.name,
        newgame.slug,
        newgame.url,
        newgame.description
    )
    .fetch_one(trans.as_mut())
    .await?
    .id;
    let category_id = query!(
        "INSERT INTO categories
        (
            game, name, description, rules,
            scoreboard
        )
        VALUES ($1, $2, $3, $4, $5) RETURNING id",
        game_id,
        newgame.cat_name,
        newgame.cat_description,
        newgame.cat_rules,
        newgame.scoreboard,
    )
    .fetch_one(trans.as_mut())
    .await?
    .id;
    query!(
        "UPDATE games SET default_category = $1 WHERE id = $2",
        category_id,
        game_id
    )
    .execute(trans.as_mut())
    .await?;
    trans.commit().await?;
    Ok(state.redirect(format!("/game/{}", newgame.slug)))
}
