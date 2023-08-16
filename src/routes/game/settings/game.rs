use axum::{
    extract::{Path, State},
    response::Html,
};

use crate::{
    id::{GameMarker, Id},
    model::{Permissions, User},
    template::BaseRenderInfo,
    AppState, Error,
};

pub async fn get(
    State(state): State<AppState>,
    Path(game_id): Path<Id<GameMarker>>,
    user: User,
    base: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    let perms_db = sqlx::query!(
        "SELECT permissions FROM permissions WHERE game_id = $1 AND user_id = $2",
        game_id.get(),
        user.id.get()
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::InsufficientPermissions)?;
    let perms = Permissions::new(perms_db.permissions);
    if !perms.contains(Permissions::ADMINISTRATOR) {
        
    }
    Ok(Html(state.tera.render(
        "edit_game.jinja",
        &tera::Context::from_serialize(base)?,
    )?))
}

pub async fn post() {}
