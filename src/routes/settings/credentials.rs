use axum::{extract::State, response::Redirect};

use crate::{model::User, util::ValidatedForm, AppState, Error};

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct UpdateEmailForm {
    #[garde(skip)]
    old_email: String,
    #[garde(email, length(min = crate::util::MIN_EMAIL_LEN, max = crate::util::MAX_EMAIL_LEN))]
    new_email: String,
    #[garde(length(min = crate::util::MIN_PASSWORD_LEN))]
    password: String,
}

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct UpdatePasswordForm {
    #[garde(email, length(min = crate::util::MIN_EMAIL_LEN, max = crate::util::MAX_EMAIL_LEN))]
    email: String,
    #[garde(skip)]
    old_password: String,
    #[garde(length(min = crate::util::MIN_PASSWORD_LEN))]
    new_password: String,
}

pub async fn update_password(
    State(state): State<AppState>,
    ValidatedForm(form): ValidatedForm<UpdatePasswordForm>,
) -> Result<Redirect, Error> {
    let mut trans = state.postgres.begin().await?;
    let Ok(user) =
        User::from_db_auth(&state, trans.as_mut(), form.email, form.old_password).await?
    else {
        return Ok(state.redirect("/settings?incorrect=true"));
    };
    query!(
        "UPDATE users SET password = $2 WHERE id = $1",
        user.id.get(),
        crate::util::hash_password(form.new_password.as_bytes(), &state.argon)?
    )
    .execute(trans.as_mut())
    .await?;
    trans.commit().await?;
    Ok(state.redirect("/settings"))
}

pub async fn update_email(
    State(state): State<AppState>,
    ValidatedForm(form): ValidatedForm<UpdateEmailForm>,
) -> Result<Redirect, Error> {
    let mut trans = state.postgres.begin().await?;
    let Ok(user) =
        User::from_db_auth(&state, trans.as_mut(), form.old_email, form.password).await?
    else {
        return Ok(state.redirect("/settings?incorrect=true"));
    };
    query!(
        "UPDATE users SET email = $2 WHERE id = $1",
        user.id.get(),
        form.new_email
    )
    .execute(trans.as_mut())
    .await?;
    trans.commit().await?;
    Ok(state.redirect("/settings"))
}
