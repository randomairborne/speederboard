use axum::{extract::State, response::Redirect, Form};

use crate::{user::User, AppState, Error};

#[derive(serde::Deserialize)]
pub struct UpdateEmailForm {
    old_email: String,
    new_email: String,
    password: String,
}

#[derive(serde::Deserialize)]
pub struct UpdatePasswordForm {
    email: String,
    old_password: String,
    new_password: String,
}

pub async fn update_password(
    State(state): State<AppState>,
    Form(form): Form<UpdatePasswordForm>,
) -> Result<Redirect, Error> {
    let mut trans = state.postgres.begin().await?;
    let Ok(user) = User::from_db(&state, trans.as_mut(), form.email, form.old_password).await?
    else {
        return Ok(Redirect::to("?incorrect=true"));
    };
    query!(
        "UPDATE users SET password = $2 WHERE id = $1",
        user.id.get(),
        crate::utils::hash_password(form.new_password.as_bytes(), &state.argon)?
    )
    .execute(trans.as_mut())
    .await?;
    trans.commit().await?;
    Ok(Redirect::to(super::UPDATE_COMPLETE_URL))
}

pub async fn update_email(
    State(state): State<AppState>,
    Form(form): Form<UpdateEmailForm>,
) -> Result<Redirect, Error> {
    let mut trans = state.postgres.begin().await?;
    let Ok(user) = User::from_db(&state, trans.as_mut(), form.old_email, form.password).await?
    else {
        return Ok(Redirect::to("?incorrect=true"));
    };
    query!(
        "UPDATE users SET email = $2 WHERE id = $1",
        user.id.get(),
        form.new_email
    )
    .execute(trans.as_mut())
    .await?;
    trans.commit().await?;
    Ok(Redirect::to(super::UPDATE_COMPLETE_URL))
}
