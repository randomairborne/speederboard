use super::User;
use crate::{
    id::{ForumCommentMarker, ForumPostMarker, GameMarker, Id},
    AppState, Error,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ForumPost {
    pub id: Id<ForumPostMarker>,
    pub game: Id<GameMarker>,
    pub title: String,
    pub author: User,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub edited_at: Option<chrono::NaiveDateTime>,
    pub flags: i64,
}

impl ForumPost {
    pub async fn from_db(state: &AppState, id: Id<ForumPostMarker>) -> Result<Self, Error> {
        let post = query!(
            "SELECT forum_posts.id as forum_post_id,
            forum_posts.title as forum_post_title,
            forum_posts.content as forum_post_content,
            forum_posts.flags as forum_post_flags,
            forum_posts.game as forum_post_game,
            forum_posts.created_at as forum_post_created_at,
            forum_posts.edited_at as forum_post_edited_at,
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
            FROM forum_posts
            JOIN users ON forum_posts.author = users.id
            WHERE forum_posts.id = $1",
            id.get()
        )
        .fetch_one(&state.postgres)
        .await?;
        let author = User {
            id: Id::new(post.user_id),
            username: post.user_username,
            stylesheet: post.user_stylesheet,
            biography: post.user_biography,
            pfp: post.user_pfp,
            banner: post.user_banner,
            admin: post.user_admin,
            created_at: post.user_created_at,
            flags: post.user_flags,
            language: post.user_language,
        };
        Ok(Self {
            id,
            game: Id::new(post.forum_post_game),
            title: post.forum_post_title,
            author,
            content: post.forum_post_content,
            created_at: post.forum_post_created_at,
            edited_at: post.forum_post_edited_at,
            flags: post.forum_post_flags,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ForumComment {
    pub id: Id<ForumCommentMarker>,
    pub parent: Id<ForumPostMarker>,
    pub game: Id<GameMarker>,
    pub author: User,
    pub content: String,
    pub created_at: chrono::NaiveDateTime,
    pub edited_at: Option<chrono::NaiveDateTime>,
    pub flags: i64,
}
