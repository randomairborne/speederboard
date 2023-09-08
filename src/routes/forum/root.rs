use axum::extract::{Path, State};

use crate::{
    id::{ForumEntryMarker, Id, UserMarker},
    model::{Game, ForumPost},
    template::BaseRenderInfo,
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize, Clone, Debug)]
pub struct ForumPage {
    #[serde(flatten)]
    core: BaseRenderInfo,
    posts: Vec<ForumPost>,
    game: Game
}

pub async fn get(
    State(state): State<AppState>,
    core: BaseRenderInfo,
    Path(game_slug): Path<String>,
) -> HandlerResult {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let post_records = query!(
        "SELECT id, title, author, content, flags
            FROM forum_entries
            WHERE game = $1 AND parent <> NULL
            ORDER BY created_at",
        game.id.get()
    )
    .fetch_all(&state.postgres)
    .await?;
    let mut posts = Vec::with_capacity(post_records.len());
    for row in post_records {
        let id: Id<ForumEntryMarker> = Id::new(row.id);
        let author: Id<UserMarker> = Id::new(row.author);
        let title = row.title.ok_or(Error::NoTitleForRootPost)?;
        posts.push(ForumPost {
            id,
            title,
            author,
            content: row.content,
            flags: row.flags,
        });
    }
    let data = ForumPage { core, posts, game };
    state.render("forum.jinja", data)
}
