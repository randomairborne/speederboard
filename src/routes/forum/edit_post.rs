use axum::{
    extract::{Path, State},
    response::Redirect,
};

use crate::{
    id::{ForumEntryMarker, Id},
    model::{Game, Permissions, User},
    template::BaseRenderInfo,
    util::{game_n_member, ValidatedForm},
    AppState, Error,
};

#[derive(serde::Serialize, Debug, Clone)]
pub struct PostCreatePage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    user: User,
    game: Game,
}

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct PostEditForm {
    #[garde(length(max = crate::util::MAX_FORUM_TITLE_LEN, min = crate::util::MIN_FORUM_TITLE_LEN))]
    title: Option<String>,
    #[garde(length(max = crate::util::MAX_FORUM_POST_LEN, min = crate::util::MIN_FORUM_POST_LEN))]
    content: String,
}

pub async fn edit(
    State(state): State<AppState>,
    user: User,
    Path((game_slug, post_id)): Path<(String, Id<ForumEntryMarker>)>,
    ValidatedForm(form): ValidatedForm<PostEditForm>,
) -> Result<Redirect, Error> {
    let (_game, member) = game_n_member(&state, user, &game_slug).await?;
    let post = query!("SELECT * FROM forum_entries WHERE id = $1", post_id.get())
        .fetch_one(&state.postgres)
        .await?;

    if post.author != member.user.id.get() {
        return Err(Error::InsufficientPermissions);
    }

    // only allow users to set a new title if the entry's title existed
    let mut new_title = None;
    if post.title.is_some() {
        new_title = form.title;
    }

    query!(
        "UPDATE forum_entries \
        SET content = $1, title = $2, edited_at = NOW() \
        WHERE id = $3",
        form.content,
        new_title,
        post_id.get()
    )
    .execute(&state.postgres)
    .await?;

    let post_id = post.id;
    Ok(state.redirect(format!("/forum/{game_slug}/post/{post_id}")))
}

pub async fn delete(
    State(state): State<AppState>,
    user: User,
    Path((game_slug, id)): Path<(String, Id<ForumEntryMarker>)>,
) -> Result<Redirect, Error> {
    let (_game, member) = game_n_member(&state, user, &game_slug).await?;
    let post = query!("SELECT * FROM forum_entries WHERE id = $1", id.get())
        .fetch_one(&state.postgres)
        .await?;
    if post.author != member.user.id.get() && !member.perms.contains(Permissions::FORUM_MODERATOR) {
        return Err(Error::InsufficientPermissions);
    }
    query!(
        "DELETE FROM forum_entries WHERE id = $1 OR parent = $1",
        post.id,
    )
    .execute(&state.postgres)
    .await?;
    if let Some(parent) = post.parent {
        Ok(state.redirect(format!("/forum/{game_slug}/post/{parent}")))
    } else {
        Ok(state.redirect(format!("/forum/{game_slug}")))
    }
}
