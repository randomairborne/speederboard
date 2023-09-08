use axum::{
    extract::{Path, State},
    response::Redirect,
};

use crate::{
    id::{CategoryMarker, Id},
    model::{Category, Game, User},
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
    #[garde(length())]
    title: String,
}

pub async fn get(
    State(state): State<AppState>,
    user: User,
    base: BaseRenderInfo,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
) -> HandlerResult {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let category = Category::from_db(state.clone(), category_id).await?;
    if category.game != game.id {
        return Err(Error::InvalidGameCategoryPair);
    }
    let context = GetRunCreatePage {
        base,
        user,
        game,
        category,
    };
    state.render("create_run.jinja", context)
}

#[allow(clippy::unused_async)]
pub async fn create(
    State(state): State<AppState>,
    user: User,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    ValidatedForm(form): ValidatedForm<RunCreateForm>,
) -> Result<Redirect, Error> {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let category = Category::from_db(state.clone(), category_id).await?;
    if category.game != game.id {
        return Err(Error::InvalidGameCategoryPair);
    }
    if category.scoreboard {
        if form.score == 0 {
            return Err(Error::CustomFormValidation(
                "score must be filled when the leaderboard is a scoreboard".to_string(),
            ));
        }
    } else if form.consolidate_times() == 0 {
        return Err(Error::CustomFormValidation(
            "time must be filled when the leaderboard is a speedrun".to_string(),
        ));
    }
    let (score, time) = (form.score, form.consolidate_times());
    let run_id = query!(
        "INSERT INTO runs
        (
            game, category, submitter, video,
            description, score, time, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 0)
        RETURNING id",
        game.id.get(),
        category.id.get(),
        user.id.get(),
        form.video,
        form.description,
        score,
        time
    )
    .fetch_one(&state.postgres)
    .await?
    .id;
    Ok(Redirect::to(&format!(
        "{}/game/{game_slug}/category/{category_id}/run/{run_id}",
        state.config.root_url
    )))
}
