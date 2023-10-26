use axum::body::Bytes;
use axum::{extract::State, response::Redirect};
use axum_extra::extract::multipart::Multipart;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;

use crate::util::MAX_CSS_LEN;
use crate::{
    model::{User, UserUpdate},
    AppState, Error,
};

pub async fn pfp(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    let (_ctype, bytes) = multipart_into_bytes(multipart, "pfp").await?;
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
    let (_ctype, bytes) = multipart_into_bytes(multipart, "banner").await?;

    let img_data = image::load_from_memory(&bytes)?;
    let jpeg_buf: Vec<u8> = Vec::with_capacity(1024 * 1024);
    img_data.write_with_encoder(JpegEncoder::new()?)?;
    state.put_r2_file(dest, &bytes, &content_type).await?;
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
    let (content_type, bytes) = multipart_into_bytes(multipart, "stylesheet").await?;
    if bytes.len() > MAX_CSS_LEN {
        return Err(Error::CustomFormValidation(format!(
            "stylesheet too large (must be less then {MAX_CSS_LEN} bytes)"
        )));
    }
    state
        .put_r2_file(&user.stylesheet_path(), &bytes, &content_type)
        .await?;
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

async fn multipart_into_bytes(
    mut multipart: Multipart,
    target_name: &str,
) -> Result<(String, Bytes), Error> {
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
        return Ok((content_type, bytes));
    }
    Err(Error::CustomFormValidation(format!(
        "Missing field {target_name}"
    )))
}
