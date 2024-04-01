use axum::{body::Bytes, extract::State, response::Redirect};
use axum_extra::extract::multipart::Multipart;
use image::{codecs::jpeg::JpegEncoder, DynamicImage};
use tokio::{join, try_join};
use webp::PixelLayout;

use crate::{
    error::ImageTooBig,
    model::{User, UserUpdate},
    util::{ImageSizeLimit, MAX_BANNER, MAX_CSS_LEN, MAX_PFP},
    AppState, Error,
};

pub async fn pfp(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    let (_ctype, bytes) = multipart_into_bytes(multipart, "pfp").await?;
    upload_image(&state, &user, bytes, MAX_PFP, User::pfp_path).await?;
    UserUpdate::new(user.id).pfp(true).execute(&state).await?;
    Ok(state.redirect("/settings"))
}

pub async fn pfp_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    UserUpdate::new(user.id).pfp(false).execute(&state).await?;
    let (webp_path, jpeg_path) = (user.pfp_path("webp"), user.pfp_path("jpeg"));
    let (webp, jpeg) = join!(
        state.delete_r2_file(&webp_path),
        state.delete_r2_file(&jpeg_path)
    );
    webp?;
    jpeg?;
    Ok(state.redirect("/settings"))
}

pub async fn banner(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    let (_ctype, bytes) = multipart_into_bytes(multipart, "banner").await?;
    upload_image(&state, &user, bytes, MAX_BANNER, User::banner_path).await?;
    UserUpdate::new(user.id)
        .banner(true)
        .execute(&state)
        .await?;
    Ok(state.redirect("/settings"))
}

pub async fn banner_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    let update = UserUpdate::new(user.id).banner(false);
    update.execute(&state).await?;
    let (webp_path, jpeg_path) = (user.pfp_path("webp"), user.pfp_path("jpeg"));
    let (webp, jpeg) = join!(
        state.delete_r2_file(&webp_path),
        state.delete_r2_file(&jpeg_path)
    );
    webp?;
    jpeg?;
    Ok(state.redirect("/settings"))
}

pub async fn stylesheet(
    State(state): State<AppState>,
    user: User,
    multipart: Multipart,
) -> Result<Redirect, Error> {
    let (_ctype, bytes) = multipart_into_bytes(multipart, "stylesheet").await?;
    if bytes.len() > MAX_CSS_LEN {
        return Err(Error::CustomFormValidation(format!(
            "stylesheet too large (must be less then {MAX_CSS_LEN} bytes)"
        )));
    }
    state
        .put_r2_file(&user.stylesheet_path(), &bytes, "text/css")
        .await?;
    UserUpdate::new(user.id)
        .stylesheet(true)
        .execute(&state)
        .await?;
    Ok(state.redirect("/settings"))
}

pub async fn stylesheet_del(State(state): State<AppState>, user: User) -> Result<Redirect, Error> {
    state.delete_r2_file(&user.stylesheet_path()).await?;
    UserUpdate::new(user.id)
        .stylesheet(false)
        .execute(&state)
        .await?;
    Ok(state.redirect("/settings"))
}

async fn upload_image(
    state: &AppState,
    user: &User,
    bytes: Bytes,
    limit: ImageSizeLimit,
    path: fn(&User, &'static str) -> String,
) -> Result<(), Error> {
    let reencoding = state
        .spawn_rayon(
            |_s, (bytes, limit)| reencode_image(&bytes, limit),
            (bytes, limit),
        )
        .await??;
    let (webp_path, jpeg_path) = (path(user, "webp"), path(user, "jpeg"));
    try_join!(
        state.put_r2_file(&webp_path, &reencoding.webp, "image/webp"),
        state.put_r2_file(&jpeg_path, &reencoding.jpeg, "image/jpeg")
    )?;
    Ok(())
}

fn reencode_image(bytes: &Bytes, limit: ImageSizeLimit) -> Result<ImageReencoding, Error> {
    let image_data = image::load_from_memory(bytes)?;
    if image_data.height() > limit.height {
        return Err(Error::ImageTooTall(ImageTooBig {
            actual: image_data.height(),
            max: limit.height,
        }));
    }
    if image_data.width() > limit.width {
        return Err(Error::ImageTooWide(ImageTooBig {
            actual: image_data.width(),
            max: limit.width,
        }));
    }
    let jpeg = encode_jpeg(&image_data)?;
    let webp = encode_webp(&image_data);
    Ok(ImageReencoding { jpeg, webp })
}

struct ImageReencoding {
    jpeg: Vec<u8>,
    webp: Vec<u8>,
}

fn encode_jpeg(image_data: &DynamicImage) -> Result<Vec<u8>, Error> {
    let mut data: Vec<u8> = Vec::with_capacity(1024 * 1024);
    image_data.write_with_encoder(JpegEncoder::new(&mut data))?;
    Ok(data)
}

fn encode_webp(image_data: &DynamicImage) -> Vec<u8> {
    let new_image = image_data.clone().into_rgba8();
    let encoder = webp::Encoder::new(
        &*new_image,
        PixelLayout::Rgba,
        image_data.width(),
        image_data.height(),
    );
    encoder.encode(80.0).to_vec()
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
