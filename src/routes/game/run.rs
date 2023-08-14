use axum::{
    extract::{Path, State},
    response::Html,
};

use crate::{
    id::{CategoryMarker, Id, RunMarker},
    model::{Category, Game, ResolvedRun, User},
    template::BaseRenderInfo,
    AppState, Error,
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

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    Path((game_slug, category_id, run_id)): Path<(String, Id<CategoryMarker>, Id<RunMarker>)>,
    base: BaseRenderInfo,
) -> Result<Html<String>, Error> {
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
    let context_ser = tera::Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("run.jinja", &context_ser)?))
}
