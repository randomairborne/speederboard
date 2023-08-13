use axum::{
    extract::{Path, State},
    response::Html,
};
use tera::Context;

use crate::{
    id::{CategoryMarker, GameMarker, Id},
    model::{Category, Game, Run},
    template::BaseRenderInfo,
    AppState, Error,
};

#[derive(serde::Serialize)]
pub struct MiniCategory {
    name: String,
    id: Id<CategoryMarker>,
    game: Id<GameMarker>,
    sort_ascending: bool,
    sort_by_score: bool,
}

#[derive(serde::Serialize)]
pub struct RunWithName {
    #[serde(flatten)]
    base: Run,
    submitter_name: String,
}

#[derive(serde::Serialize)]
pub struct GetGameContext {
    #[serde(flatten)]
    core: BaseRenderInfo,
    categories: Vec<MiniCategory>,
    category: Category,
    runs: Vec<RunWithName>,
    game: Game,
}

pub async fn get(
    State(state): State<AppState>,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    core: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    get_game_category(&state, core, game_slug, Some(category_id)).await
}

pub(super) async fn get_game_category(
    state: &AppState,
    core: BaseRenderInfo,
    game_slug: String,
    maybe_category_id: Option<Id<CategoryMarker>>,
) -> Result<Html<String>, Error> {
    let game = Game::from_db_slug(state, &game_slug).await?;
    let category_id = maybe_category_id.unwrap_or(game.default_category);
    let state2 = state.clone();
    let spawned_getcats = tokio::spawn(async move {
        query_as!(
            MiniCategory,
            "SELECT name, id, game, sort_ascending, sort_by_score
        FROM categories WHERE game = $1",
            game.id.get()
        )
        .fetch_all(&state2.postgres)
        .await
    });
    let category = query_as!(
        Category,
        "SELECT * FROM categories WHERE id = $1",
        category_id.get()
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let order_by = format!(
        "{}{}",
        if category.sort_ascending { "-" } else { "+" },
        if category.sort_by_score {
            "score"
        } else {
            "time"
        }
    );
    let runs = query!(
        "SELECT runs.*, users.username as submitter_name
        FROM runs JOIN users
        ON runs.submitter = users.id
        WHERE game = $1 AND category = $2
        ORDER BY $3 LIMIT 51",
        game.id.get(),
        category.id.get(),
        order_by
    )
    .fetch_all(&state.postgres)
    .await?;
    let runs = runs
        .into_iter()
        .map(|r| RunWithName {
            submitter_name: r.submitter_name,
            base: Run {
                id: r.id.into(),
                game: r.game.into(),
                category: r.category.into(),
                submitter: r.submitter.into(),
                verifier: r.verifier.map(Into::into),
                video: r.video,
                description: r.description,
                score: r.score,
                time: r.time,
                status: r.status.into(),
            },
        })
        .collect();
    let categories = spawned_getcats.await??;
    let get_game_ctx = GetGameContext {
        core,
        categories,
        category,
        runs,
        game,
    };
    let ctx = Context::from_serialize(get_game_ctx)?;
    Ok(Html(state.tera.render("game.jinja", &ctx)?))
}
