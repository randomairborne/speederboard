use axum::{
    extract::{Path, State},
    response::Redirect,
};

use crate::{
    id::{ForumCommentMarker, ForumPostMarker, GameMarker, Id},
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
    Path((game_slug, post_id)): Path<(String, Id<ForumPostMarker>)>,
) -> HandlerResult {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let post = ForumPost::from_db(&state, post_id).await?;
    let runs = query!(
        "SELECT forum_comments.id as forum_comment_id,
        forum_comments.game as forum_comment_game,
        forum_comments.content as forum_comment_content,
        forum_comments.flags as forum_comment_flags,
        forum_comments.parent as forum_comment_parent,
        forum_comments.created_at as forum_comment_created_at,
        forum_comments.edited_at as forum_comment_edited_at,
        users.id as user_id,
        users.username as user_username,
        users.biography as user_biography,
        users.admin as user_admin,
        users.stylesheet as user_stylesheet,
        users.banner as user_banner,
        users.pfp as user_pfp,
        users.flags as user_flags,
        users.created_at as user_created_at,
        users.language as user_language
        FROM forum_comments
        JOIN users ON forum_comments.author = users.id
        WHERE forum_comments.id = $1 OR forum_comments.parent = $1
        ORDER BY forum_comments.created_at",
        post_id.get()
    )
    .fetch_all(&state.postgres)
    .await?;
    let mut comments: Vec<ForumComment> = Vec::with_capacity(runs.len());
    for run in runs {
        let author = User {
            id: Id::new(run.user_id),
            username: run.user_username,
            stylesheet: run.user_stylesheet,
            biography: run.user_biography,
            pfp: run.user_pfp,
            banner: run.user_banner,
            admin: run.user_admin,
            created_at: run.user_created_at,
            flags: run.user_flags,
            language: run.user_language,
        };
        let id: Id<ForumCommentMarker> = Id::new(run.forum_comment_id);
        let parent: Id<ForumPostMarker> = Id::new(run.forum_comment_parent);
        let game: Id<GameMarker> = Id::new(run.forum_comment_game);
        let content = run.forum_comment_content;
        let created_at = run.forum_comment_created_at;
        let edited_at = run.forum_comment_edited_at;
        let flags = run.forum_comment_flags;
        comments.push(ForumComment {
            id,
            parent,
            game,
            author,
            content,
            created_at,
            edited_at,
            flags,
        });
    }
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
    Path((game_slug, post_id)): Path<(String, Id<ForumPostMarker>)>,
    ValidatedForm(form): ValidatedForm<CommentCreateForm>,
) -> Result<Redirect, Error> {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let id = query!(
        "INSERT INTO forum_comments (parent, game, author, content, created_at)
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
