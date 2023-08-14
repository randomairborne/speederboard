use axum::{
    extract::{Path, State},
    response::Html,
};

use crate::{
    id::{CategoryMarker, Id, RunMarker},
    model::{Category, Game, ResolvedRun, Run, User},
    template::BaseRenderInfo,
    AppState, Error,
};

#[derive(serde::Serialize)]
pub struct RunPage {
    run: ResolvedRun,
    user: User,
    game: Game,
    category: Category,
    verifier: Option<User>,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

#[allow(clippy::unused_async)]
pub async fn get(
    State(state): State<AppState>,
    Path((game_slug, category_id, run_id)): Path<(String, Id<CategoryMarker>, Id<RunMarker>)>,
    base: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    let record = query!("SELECT * FROM runs WHERE id = $1", run_id.get())
        .fetch_optional(&state.postgres)
        .await?
        .ok_or(Error::NotFound)?;
    let user = User::from_db(&state, Id::new(record.submitter)).await?;
    if game.slug != game_slug || category.id != category_id {
        return Err(Error::NotFound);
    }
    let run = Run {
        id: record.id.into(),
        game: record.game.into(),
        category: record.category.into(),
        submitter: record.submitter.into(),
        verifier: record.verifier.map(Into::into),
        video: record.video,
        description: record.description,
        score: record.score,
        time: record.time,
        status: record.status.into(),
    };
    let ctx = RunPage {
        run,
        user,
        game,
        category,
        verifier,
        base,
    };
    let context_ser = tera::Context::from_serialize(ctx)?;
    Ok(Html(state.tera.render("run.jinja", &context_ser)?))
}
