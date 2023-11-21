use axum::{
    extract::{Path, State},
    response::Redirect,
};

use crate::{
    id::{ForumCommentMarker, ForumPostMarker, Id},
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
    title: String,
    #[garde(length(max = crate::util::MAX_FORUM_POST_LEN, min = crate::util::MIN_FORUM_POST_LEN))]
    content: String,
}

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct CommentEditForm {
    #[garde(length(max = crate::util::MAX_FORUM_POST_LEN, min = crate::util::MIN_FORUM_POST_LEN))]
    content: String,
}

pub async fn edit_post(
    State(state): State<AppState>,
    user: User,
    Path((game_slug, post_id)): Path<(String, Id<ForumPostMarker>)>,
    ValidatedForm(form): ValidatedForm<PostEditForm>,
) -> Result<Redirect, Error> {
    let (_game, member) = game_n_member(&state, user, &game_slug).await?;
    let post = query!("SELECT * FROM forum_posts WHERE id = $1", post_id.get())
        .fetch_one(&state.postgres)
        .await?;

    if post.author != member.user.id.get() {
        return Err(Error::InsufficientPermissions);
    }
    query!(
        "UPDATE forum_posts \
        SET content = $1, title = $2, edited_at = NOW() \
        WHERE id = $3",
        form.content,
        form.title,
        post_id.get()
    )
    .execute(&state.postgres)
    .await?;
    let post_id = post.id;
    Ok(state.redirect(format!("/forum/{game_slug}/post/{post_id}")))
}

pub async fn edit_comment(
    State(state): State<AppState>,
    user: User,
    Path((game_slug, post_id)): Path<(String, Id<ForumCommentMarker>)>,
    ValidatedForm(form): ValidatedForm<CommentEditForm>,
) -> Result<Redirect, Error> {
    let (_game, member) = game_n_member(&state, user, &game_slug).await?;
    let post = query!("SELECT * FROM forum_comments WHERE id = $1", post_id.get())
        .fetch_one(&state.postgres)
        .await?;
    if post.author != member.user.id.get() {
        return Err(Error::InsufficientPermissions);
    }
    query!(
        "UPDATE forum_posts \
        SET content = $1, edited_at = NOW() \
        WHERE id = $2",
        form.content,
        post_id.get()
    )
    .execute(&state.postgres)
    .await?;
    let post_id = post.id;
    Ok(state.redirect(format!("/forum/{game_slug}/post/{post_id}")))
}

pub async fn delete_post(
    State(state): State<AppState>,
    user: User,
    Path((game_slug, id)): Path<(String, Id<ForumPostMarker>)>,
) -> Result<Redirect, Error> {
    let (_game, member) = game_n_member(&state, user, &game_slug).await?;
    let post = query!("SELECT * FROM forum_posts WHERE id = $1", id.get())
        .fetch_one(&state.postgres)
        .await?;
    if post.author != member.user.id.get() && !member.perms.contains(Permissions::FORUM_MODERATOR) {
        return Err(Error::InsufficientPermissions);
    }
    query!("DELETE FROM forum_posts WHERE id = $1", post.id,)
        .execute(&state.postgres)
        .await?;
    query!("DELETE FROM forum_comments WHERE parent = $1", post.id,)
        .execute(&state.postgres)
        .await?;
    Ok(state.redirect(format!("/forum/{game_slug}")))
}

pub async fn delete_comment(
    State(state): State<AppState>,
    user: User,
    Path((game_slug, id)): Path<(String, Id<ForumCommentMarker>)>,
) -> Result<Redirect, Error> {
    let (_game, member) = game_n_member(&state, user, &game_slug).await?;
    let post = query!("SELECT * FROM forum_comments WHERE id = $1", id.get())
        .fetch_one(&state.postgres)
        .await?;
    if post.author != member.user.id.get() && !member.perms.contains(Permissions::FORUM_MODERATOR) {
        return Err(Error::InsufficientPermissions);
    }
    query!("DELETE FROM forum_comments WHERE id = $1", post.id,)
        .execute(&state.postgres)
        .await?;
    let parent = post.parent;
    Ok(state.redirect(format!("/forum/{game_slug}/post/{parent}")))
}
