use axum::{
    extract::{Path, State},
    response::Redirect,
};

use crate::{
    id::{CategoryMarker, Id, RunMarker},
    model::{Category, Game, Permissions, ResolvedRun, User},
    template::BaseRenderInfo,
    util::game_n_member,
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize, Debug, Clone)]
pub struct RunPage<'a> {
    pub user: &'a User,
    pub game: &'a Game,
    pub category: &'a Category,
    pub verifier: &'a Option<User>,
    pub run: &'a ResolvedRun,
    #[serde(flatten)]
    pub base: BaseRenderInfo,
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

pub async fn delete(
    State(state): State<AppState>,
    Path((game_slug, _category_id, run_id)): Path<(String, Id<CategoryMarker>, Id<RunMarker>)>,
    user: User,
) -> Result<Redirect, Error> {
    let (_game, member) = game_n_member(&state, user, &game_slug).await?;
    let run = query!("SELECT submitter FROM runs WHERE id = $1", run_id.get())
        .fetch_optional(&state.postgres)
        .await?
        .ok_or(Error::NotFound)?;
    if !member.perms.contains(Permissions::LEADERBOARD_MODERATOR)
        && member.user.id.get() != run.submitter
    {
        return Err(Error::InsufficientPermissions);
    }
    query!("DELETE FROM runs WHERE id = $1", run_id.get())
        .execute(&state.postgres)
        .await?;
    Ok(state.redirect(format!("/game/{game_slug}")))
}
