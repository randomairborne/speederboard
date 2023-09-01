use axum::{
    extract::{Path, State},
    response::{Html, Redirect},
};
use garde::Validate;

use crate::{
    id::{CategoryMarker, Id},
    model::{Category, Game, User},
    template::BaseRenderInfo,
    util::ValidatedForm,
    AppState, Error,
};

#[derive(serde::Serialize)]
pub struct GetRunCreatePage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    user: User,
    game: Game,
    category: Category,
}

#[derive(serde::Deserialize, Validate)]
pub struct RunCreateForm {
    #[garde(length(min = crate::util::MIN_RUN_VIDEO_LEN, max = crate::util::MAX_RUN_VIDEO_LEN))]
    video: String,
    #[garde(length(min = crate::util::MIN_RUN_DESCRIPTION_LEN, max = crate::util::MAX_RUN_DESCRIPTION_LEN))]
    description: String,
    #[garde(range(min = 0))]
    score: Option<i64>,
    #[garde(range(min = 0))]
    time: Option<i64>,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    user: User,
    base: BaseRenderInfo,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
) -> Result<Html<String>, Error> {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let category = Category::from_db(state.clone(), category_id).await?;
    if category.game != game.id {
        return Err(Error::InvalidGameCategoryPair);
    }
    let struct_context = GetRunCreatePage {
        base,
        user,
        game,
        category,
    };
    let ctx = tera::Context::from_serialize(struct_context)?;
    Ok(Html(state.tera.render("create_run.jinja", &ctx)?))
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
        if form.score.is_none() {
            return Err(Error::CustomFormValidation(
                "score must be filled when the leaderboard is a scoreboard".to_string(),
            ));
        }
    } else if form.time.is_none() {
        return Err(Error::CustomFormValidation(
            "time must be filled when the leaderboard is a speedrun".to_string(),
        ));
    }
    let (score, time) = (form.score.unwrap_or(0), form.time.unwrap_or(0));
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
