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

fn ret_0() -> i64 {
    0
}

#[derive(serde::Deserialize, Validate)]
pub struct RunCreateForm {
    #[garde(length(min = crate::util::MIN_RUN_VIDEO_LEN, max = crate::util::MAX_RUN_VIDEO_LEN))]
    video: String,
    #[garde(length(min = crate::util::MIN_RUN_DESCRIPTION_LEN, max = crate::util::MAX_RUN_DESCRIPTION_LEN))]
    description: String,
    #[serde(default = "ret_0")]
    #[garde(range(min = 0))]
    score: i64,
    #[serde(default = "ret_0")]
    #[garde(range(min = 0))]
    hours: i64,
    #[serde(default = "ret_0")]
    #[garde(range(min = 0, max = 60))]
    minutes: i64,
    #[serde(default = "ret_0")]
    #[garde(range(min = 0, max = 60))]
    seconds: i64,
    #[serde(default = "ret_0")]
    #[garde(range(min = 0, max = 1000))]
    milliseconds: i64,
}

impl RunCreateForm {
    const MS_PER_HOUR: i64 = 3_600_000;
    const MS_PER_MINUTE: i64 = 60_000;
    const MS_PER_SECOND: i64 = 1000;
    pub fn consolidate_times(&self) -> i64 {
        (self.hours * Self::MS_PER_HOUR)
            + (self.minutes * Self::MS_PER_MINUTE)
            + (self.seconds * Self::MS_PER_SECOND)
            + self.milliseconds
    }
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
