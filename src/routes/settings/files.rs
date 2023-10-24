use axum::{extract::State, response::Redirect};
use axum_extra::extract::multipart::Multipart;

use crate::{
    model::{User, UserUpdate},
    AppState, Error,
};

pub async fn pfp(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    multipart_into_s3(&state, multipart, "pfp", &user.pfp_path("png")).await?;
    let update = UserUpdate::new(user.id).pfp(true);
    update.execute(&state).await?;
    Ok(state.redirect("/settings"))
}

pub async fn pfp_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.delete_r2_file(&user.pfp_path("png")).await?;
    let update = UserUpdate::new(user.id).pfp(false);
    update.execute(&state).await?;
    Ok(state.redirect("/settings"))
}

pub async fn banner(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    multipart_into_s3(&state, multipart, "banner", &user.banner_path("png")).await?;
    let update = UserUpdate::new(user.id).banner(true);
    update.execute(&state).await?;
    Ok(state.redirect("/settings"))
}

pub async fn banner_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.delete_r2_file(&user.banner_path("png")).await?;
    let update = UserUpdate::new(user.id).banner(false);
    update.execute(&state).await?;
    Ok(state.redirect("/settings"))
}

pub async fn stylesheet(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    multipart_into_s3(&state, multipart, "stylesheet", &user.stylesheet_path()).await?;
    let update = UserUpdate::new(user.id).has_stylesheet(true);
    update.execute(&state).await?;
    Ok(state.redirect("/settings"))
}

pub async fn stylesheet_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.delete_r2_file(&user.stylesheet_path()).await?;
    let update = UserUpdate::new(user.id).has_stylesheet(false);
    update.execute(&state).await?;
    Ok(state.redirect("/settings"))
}

const SIZE_LIMIT: usize = 1024 * 512;

async fn multipart_into_s3(
    state: &AppState,
    mut multipart: Multipart,
    target_name: &str,
    dest: &str,
) -> Result<(), Error> {
    while let Some(field) = multipart.next_field().await? {
        let Some(name) = field.name().map(ToOwned::to_owned) else {
            continue;
        };
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
        // todo: validate and convert images
        // let img_data = image::load_from_memory(&bytes)?;
        state
            .put_r2_file(dest, reqwest::Body::from(bytes), &content_type)
            .await?;
    }
    Ok(())
}
