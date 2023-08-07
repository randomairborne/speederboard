// pub async fn post(
//     State(state): State<AppState>,
//     user: User,
//     mpup: Multipart,
// ) -> Result<Redirect, Error> {
//     while let Some(mut mp) = mpup.next_field().await? {
//         mp.
//     }
//     query!(
//         "UPDATE users
//         SET username = COALESCE($2, username),
//         biography = COALESCE($3, username)
//         WHERE id = $1",
//         user.id.get(),
//         form.username,
//         form.biography
//     )
//     .execute(&state.postgres)
//     .await?;
//     Ok(Redirect::to("?updated=true"))
// }

#[derive(serde::Deserialize)]
pub struct UserUpdateForm {
    username: Option<String>,
    biography: Option<String>,
}

pub async fn get() -> &'static str {
    ""
}

pub async fn pfp() -> &'static str {
    ""
}

pub async fn banner() -> &'static str {
    ""
}
