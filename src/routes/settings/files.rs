use super::UPDATE_COMPLETE_URL;
use crate::{model::User, state::DbUserUpdate, AppState, Error};
use axum::{extract::State, response::Redirect};
use axum_extra::extract::multipart::Multipart;

pub async fn pfp(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    multipart_into_s3(&state, multipart, "pfp", &user.pfp_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).pfp_ext(Some("png".to_string()));
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn pfp_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.delete_r2_file(&user.pfp_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).pfp_ext(None);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn banner(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    multipart_into_s3(&state, multipart, "banner", &user.banner_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).banner_ext(Some("png".to_string()));
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn banner_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.delete_r2_file(&user.banner_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).banner_ext(None);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn stylesheet(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    multipart_into_s3(
        &state,
        multipart,
        "stylesheet",
        &user.stylesheet_dest_path(),
    )
    .await?;
    let update = DbUserUpdate::new(user.id).has_stylesheet(true);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

pub async fn stylesheet_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.delete_r2_file(&user.stylesheet_dest_path()).await?;
    let update = DbUserUpdate::new(user.id).has_stylesheet(false);
    state.update_user(update).await?;
    Ok(Redirect::to(UPDATE_COMPLETE_URL))
}

const SIZE_LIMIT: usize = 1024 * 512;

async fn multipart_into_s3(
    state: &AppState,
    mut multipart: Multipart,
    target_name: &str,
    dest: &str,
) -> Result<(), Error> {
    while let Some(field) = multipart.next_field().await? {
        if let Some(name) = field.name().map(std::string::ToString::to_string) {
            if name != target_name {
                continue;
            }
            let content_type = {
                let ctype = field.content_type();
                ctype.unwrap_or("application/octet-stream").to_string()
            };
            let bytes = field.bytes().await?;
            if bytes.len() > SIZE_LIMIT {
                return Err(Error::CustomFormValidation(format!(
                    "File was expected to be less then {SIZE_LIMIT} bytes",
                )));
            }
            state
                .put_r2_file(dest, reqwest::Body::from(bytes), &content_type)
                .await?;
        }
    }
    Ok(())
}
