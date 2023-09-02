use crate::{
    id::{CategoryMarker, Id},
    model::{Category, Game, Permissions, User},
    template::{BaseRenderInfo, ConfirmContext},
    util::{self, ValidatedForm},
    AppState, Error, HandlerResult,
};
use axum::{
    extract::{Path, State},
    response::Redirect,
};

#[derive(serde::Serialize)]
pub struct GetCategoryContext {
    #[serde(flatten)]
    base: BaseRenderInfo,
    category: Category,
    game: Game,
}

#[derive(serde::Deserialize, Clone, Debug, garde::Validate)]
pub struct NewCategory {
    #[garde(length(min = crate::util::MIN_CATEGORY_NAME_LEN, max = crate::util::MAX_CATEGORY_NAME_LEN))]
    name: String,
    #[garde(length(min = crate::util::MIN_CATEGORY_DESCRIPTION_LEN, max = crate::util::MAX_CATEGORY_DESCRIPTION_LEN))]
    description: String,
    #[garde(length(min = crate::util::MIN_CATEGORY_RULES_LEN, max = crate::util::MAX_CATEGORY_RULES_LEN))]
    rules: String,
    #[serde(default = "return_false")]
    #[garde(skip)]
    scoreboard: bool,
}

fn return_false() -> bool {
    false
}

pub async fn new(
    State(state): State<AppState>,
    Path(game_slug): Path<String>,
    user: User,
    ValidatedForm(form): ValidatedForm<NewCategory>,
) -> Result<Redirect, Error> {
    let (game, member) = util::game_n_member(&state, user, &game_slug).await?;
    member.perms.check(Permissions::ADMINISTRATOR)?;
    let cat_id = query!(
        "INSERT INTO categories (game, name, description, rules, scoreboard)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id",
        game.id.get(),
        form.name,
        form.description,
        form.rules,
        form.scoreboard
    )
    .fetch_one(&state.postgres)
    .await?
    .id;
    Ok(Redirect::to(&format!(
        "/game/{game_slug}/category/{cat_id}"
    )))
}

#[allow(clippy::unused_async)]
pub async fn confirm_delete(
    State(state): State<AppState>,
    user: User,
    base: BaseRenderInfo,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
) -> HandlerResult {
    let (_game, member) = util::game_n_member(&state, user, &game_slug).await?;
    member.perms.check(Permissions::ADMINISTRATOR)?;
    let ctx = ConfirmContext {
        base,
        action: "delete this category".to_string(),
        action_url: format!("/game/{game_slug}/delete-category/{category_id}",),
        return_to: format!("/game/{game_slug}/edit"),
    };
    state.render("confirm.jinja", ctx)
}

pub async fn delete(
    State(state): State<AppState>,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    user: User,
) -> Result<Redirect, Error> {
    let (game, member) = util::game_n_member(&state, user, &game_slug).await?;
    member.perms.check(Permissions::ADMINISTRATOR)?;
    if game.default_category == category_id {
        return Err(Error::CannotDeleteDefaultCategory);
    }
    query!(
        "DELETE FROM categories WHERE id = $1 AND game = $2",
        category_id.get(),
        game.id.get()
    )
    .execute(&state.postgres)
    .await?;
    Ok(Redirect::to(&format!("/game/{game_slug}/edit")))
}

pub async fn edit(
    State(state): State<AppState>,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    user: User,
    ValidatedForm(form): ValidatedForm<NewCategory>,
) -> Result<Redirect, Error> {
    let (game, member) = util::game_n_member(&state, user, &game_slug).await?;
    member.perms.check(Permissions::ADMINISTRATOR)?;
    println!("{form:?}");
    query!(
        "UPDATE categories
            SET name = $3, description = $4,
            rules = $5, scoreboard = $6
            WHERE id = $1 AND game = $2",
        category_id.get(),
        game.id.get(),
        form.name,
        form.description,
        form.rules,
        form.scoreboard
    )
    .execute(&state.postgres)
    .await?;
    Ok(Redirect::to(&format!(
        "/game/{game_slug}/category/{category_id}/edit"
    )))
}

pub async fn get(
    State(state): State<AppState>,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    base: BaseRenderInfo,
    user: User,
) -> HandlerResult {
    let (game, member) = util::game_n_member(&state, user, &game_slug).await?;
    member.perms.check(Permissions::ADMINISTRATOR)?;
    let category = Category::from_db(state.clone(), category_id).await?;
    let ctx = GetCategoryContext {
        base,
        category,
        game,
    };
    state.render("edit_category.jinja", ctx)
}
