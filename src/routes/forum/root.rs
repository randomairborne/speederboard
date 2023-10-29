use axum::extract::{Path, State};

use crate::{
    id::{ForumEntryMarker, Id},
    model::{ForumPost, Game, User},
    template::BaseRenderInfo,
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize, Clone, Debug)]
pub struct ForumPage {
    #[serde(flatten)]
    base: BaseRenderInfo,
    posts: Vec<ForumPost>,
    game: Game,
}

pub async fn get(
    State(state): State<AppState>,
    base: BaseRenderInfo,
    Path(game_slug): Path<String>,
) -> HandlerResult {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let post_records = query!(
        "SELECT forum_entries.id as forum_entry_id,
            forum_entries.title as forum_entry_title,
            forum_entries.content as forum_entry_content,
            forum_entries.flags as forum_entry_flags,
            forum_entries.created_at as forum_entry_created_at,
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
            FROM forum_entries
            JOIN users ON forum_entries.author = users.id
            WHERE game = $1 AND title IS NOT NULL",
        game.id.get()
    )
    .fetch_all(&state.postgres)
    .await?;
    let mut posts = Vec::with_capacity(post_records.len());
    for row in post_records {
        let id: Id<ForumEntryMarker> = Id::new(row.forum_entry_id);
        let title = row.forum_entry_title.ok_or(Error::NoTitleForRootPost)?;
        let author = User {
            id: Id::new(row.user_id),
            username: row.user_username,
            stylesheet: row.user_stylesheet,
            biography: row.user_biography,
            pfp: row.user_pfp,
            banner: row.user_banner,
            admin: row.user_admin,
            created_at: row.user_created_at,
            flags: row.user_flags,
            language: row.user_language,
        };
        posts.push(ForumPost {
            id,
            title,
            author,
            content: row.forum_entry_content,
            created_at: row.forum_entry_created_at,
            flags: row.forum_entry_flags,
        });
    }
    let data = ForumPage { base, posts, game };
    state.render("forum.jinja", data)
}
