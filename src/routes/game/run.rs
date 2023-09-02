use axum::extract::{Path, State};

use crate::{
    id::{CategoryMarker, Id, RunMarker},
    model::{Category, Game, ResolvedRun, User},
    template::BaseRenderInfo,
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize)]
pub struct RunPage<'a> {
    user: &'a User,
    game: &'a Game,
    category: &'a Category,
    verifier: &'a Option<User>,
    run: &'a ResolvedRun,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

pub async fn get(
    State(state): State<AppState>,
    Path((game_slug, category_id, run_id)): Path<(String, Id<CategoryMarker>, Id<RunMarker>)>,
    base: BaseRenderInfo,
) -> HandlerResult {
    let run = ResolvedRun::from_db(&state, run_id)
        .await?
        .ok_or(Error::NotFound)?;
    if run.game.slug != game_slug || run.category.id != category_id {
        return Err(Error::NotFound);
    }
    let ctx = RunPage {
        user: &run.submitter,
        game: &run.game,
        category: &run.category,
        verifier: &run.verifier,
        run: &run,
        base,
    };
    state.render("run.jinja", ctx)
}
