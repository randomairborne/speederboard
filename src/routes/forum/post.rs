use axum::{
    extract::{Path, State},
    response::Redirect,
};

use crate::{
    id::{ForumEntryMarker, Id},
    model::{ForumComment, ForumPost, Game, User},
    template::BaseRenderInfo,
    util::ValidatedForm,
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize, Clone, Debug)]
pub struct ForumPostPage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    comments: Vec<ForumComment>,
    post: ForumPost,
    game: Game,
}

#[derive(serde::Deserialize, garde::Validate, Clone, Debug)]
pub struct CommentCreateForm {
    #[garde(length(max = crate::util::MAX_FORUM_COMMENT_LEN, min = crate::util::MIN_FORUM_COMMENT_LEN))]
    content: String,
}

pub async fn get(
    State(state): State<AppState>,
    base: BaseRenderInfo,
    Path((game_slug, post_id)): Path<(String, Id<ForumEntryMarker>)>,
) -> HandlerResult {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let runs = query!(
        "SELECT forum_entries.id as forum_entry_id,
        forum_entries.title as forum_entry_title,
        forum_entries.content as forum_entry_content,
        forum_entries.flags as forum_entry_flags,
        forum_entries.parent as forum_entry_parent,
        forum_entries.created_at as forum_entry_created_at,
        users.id as user_id,
        users.username as user_username,
        users.biography as user_biography,
        users.admin as user_admin,
        users.has_stylesheet as user_has_stylesheet,
        users.banner_ext as user_banner_ext,
        users.pfp_ext as user_pfp_ext,
        users.flags as user_flags,
        users.created_at as user_created_at,
        users.language as user_language
        FROM forum_entries
        JOIN users ON forum_entries.author = users.id
        WHERE forum_entries.id = $1 OR forum_entries.parent = $1
        ORDER BY forum_entries.created_at",
        post_id.get()
    )
    .fetch_all(&state.postgres)
    .await?;
    let mut comments: Vec<ForumComment> = Vec::with_capacity(runs.len());
    let mut post: Option<ForumPost> = None;
    for run in runs {
        let author = User {
            id: Id::new(run.user_id),
            username: run.user_username,
            has_stylesheet: run.user_has_stylesheet,
            biography: run.user_biography,
            pfp_ext: run.user_pfp_ext,
            banner_ext: run.user_banner_ext,
            admin: run.user_admin,
            created_at: run.user_created_at,
            flags: run.user_flags,
            language: run.user_language,
        };
        let id: Id<ForumEntryMarker> = Id::new(run.forum_entry_id);
        let content = run.forum_entry_content;
        let created_at = run.forum_entry_created_at;
        let flags = run.forum_entry_flags;
        if let Some(forum_entry_title) = run.forum_entry_title {
            post = Some(ForumPost {
                id,
                title: forum_entry_title,
                author,
                content,
                created_at,
                flags,
            });
        } else if let Some(parent) = run.forum_entry_parent {
            comments.push(ForumComment {
                id,
                parent: Id::new(parent),
                author,
                content,
                created_at,
                flags,
            });
        }
    }
    let Some(post) = post else {
        return Err(Error::NotFound);
    };
    let page = ForumPostPage {
        base,
        comments,
        post,
        game,
    };
    state.render("forum_post.jinja", page)
}

pub async fn post(
    State(state): State<AppState>,
    user: User,
    Path((game_slug, post_id)): Path<(String, Id<ForumEntryMarker>)>,
    ValidatedForm(form): ValidatedForm<CommentCreateForm>,
) -> Result<Redirect, Error> {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let id = query!(
        "INSERT INTO forum_entries (parent, game, author, content, created_at)
            VALUES ($1, $2, $3, $4, NOW()) RETURNING id",
        post_id.get(),
        game.id.get(),
        user.id.get(),
        form.content
    )
    .fetch_one(&state.postgres)
    .await?
    .id;
    Ok(state.redirect(format!("/forum/{game_slug}/post/{post_id}#comment-{id}")))
}
