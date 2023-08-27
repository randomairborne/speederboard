use crate::{util::ValidatedForm, AppState, Error};
use axum::{
    extract::{Path, State},
    response::Redirect,
};

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
    ValidatedForm(form): ValidatedForm<NewCategory>,
) -> Result<Redirect, Error> {
    let cat_id = query!(
        "INSERT INTO categories (name, description, rules, scoreboard)
        VALUES ($1, $2, $3, $4)
        RETURNING id",
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

pub async fn confirm_delete() {}

pub async fn delete() {}

pub async fn edit() {}

pub async fn get() {}
