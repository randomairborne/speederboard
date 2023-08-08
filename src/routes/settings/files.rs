use axum::extract::Multipart;
use axum::{extract::State, response::Redirect};
use futures_util::TryStreamExt;

use super::UPDATE_COMPLETE_URL;
use crate::{state::DbUserUpdate, user::User, AppState, Error};

pub async fn pfp(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    multipart_into_s3(&state, multipart, user.pfp_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).pfp_ext(Some("png".to_string()));
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn pfp_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.s3.delete_object(user.pfp_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).pfp_ext(None);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn banner(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    multipart_into_s3(&state, multipart, user.banner_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).banner_ext(Some("png".to_string()));
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn banner_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.s3.delete_object(user.banner_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).banner_ext(None);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn stylesheet(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    multipart_into_s3(&state, multipart, user.stylesheet_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).has_stylesheet(true);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn stylesheet_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.s3.delete_object(user.stylesheet_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).has_stylesheet(false);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

async fn multipart_into_s3(
    state: &AppState,
    mut multipart: Multipart,
    dest: String,
) -> Result<(), Error> {
    let image = multipart
        .next_field()
        .await?
        .ok_or(Error::InvalidMultipart(
            "you need at least one multipart field",
        ))?;
    let mut file_stream = tokio_util::io::StreamReader::new(image.map_err(Error::Multipart));
    state.s3.put_object_stream(&mut file_stream, dest).await?;
    Ok(())
}
