use axum::{
    extract::{Path, State},
    response::Html,
};
use tera::Context;

use crate::{
    id::{CategoryMarker, GameMarker, Id},
    model::{Category, Game, ResolvedRunRef, RunStatus, User},
    template::BaseRenderInfo,
    util::opt_user,
    AppState, Error,
};

#[derive(serde::Serialize)]
pub struct MiniCategory {
    name: String,
    id: Id<CategoryMarker>,
    game: Id<GameMarker>,
    scoreboard: bool,
}

#[derive(serde::Serialize)]
pub struct GetGameContext<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo,
    categories: Vec<MiniCategory>,
    category: &'a Category,
    runs: Vec<ResolvedRunRef<'a>>,
    game: &'a Game,
}

pub async fn get(
    State(state): State<AppState>,
    Path((game_slug, category_id)): Path<(String, Id<CategoryMarker>)>,
    core: BaseRenderInfo,
) -> Result<Html<String>, Error> {
    get_game_category(&state, core, game_slug, Some(category_id)).await
}

pub(super) async fn get_game_category(
    state: &AppState,
    core: BaseRenderInfo,
    game_slug: String,
    maybe_category_id: Option<Id<CategoryMarker>>,
) -> Result<Html<String>, Error> {
    let game = Game::from_db_slug(state, &game_slug).await?;
    let category_id = maybe_category_id.unwrap_or(game.default_category);
    let state2 = state.clone();
    let spawned_getcats = tokio::spawn(async move {
        query_as!(
            MiniCategory,
            "SELECT name, id, game, scoreboard
            FROM categories WHERE game = $1",
            game.id.get()
        )
        .fetch_all(&state2.postgres)
        .await
    });
    let category = query_as!(
        Category,
        "SELECT * FROM categories WHERE id = $1",
        category_id.get()
    )
    .fetch_optional(&state.postgres)
    .await?
    .ok_or(Error::NotFound)?;
    let runs: Vec<ResolvedRunRef> = if category.scoreboard {
        get_scoreboard(state, &game, &category).await?
    } else {
        get_speedrun(state, &game, &category).await?
    };
    let categories = spawned_getcats.await??;
    let get_game_ctx = GetGameContext {
        core,
        categories,
        category: &category,
        runs,
        game: &game,
    };
    let ctx = Context::from_serialize(get_game_ctx)?;
    Ok(Html(state.tera.render("category.jinja", &ctx)?))
}

async fn get_scoreboard<'a>(
    state: &AppState,
    game: &'a Game,
    category: &'a Category,
) -> Result<Vec<ResolvedRunRef<'a>>, Error> {
    let records = query!(
        r#"SELECT runs.id, runs.game, runs.category, runs.video,
            runs.description, runs.score, runs.time, runs.status,
            runs.created_at, runs.verified_at,
            ver.id as "ver_id?", sub.id as sub_id,
            ver.username as "ver_name?", sub.username as sub_name,
            ver.has_stylesheet as "ver_has_stylesheet?",
            sub.has_stylesheet as sub_has_stylesheet,
            ver.biography as "ver_bio?", sub.biography as sub_bio,
            ver.pfp_ext as ver_pfp_ext, sub.pfp_ext as sub_pfp_ext,
            ver.banner_ext as ver_banner_ext,
            sub.banner_ext as sub_banner_ext,
            ver.admin as "ver_admin?", sub.admin as sub_admin,
            ver.created_at as "ver_created_at?",
            sub.created_at as sub_created_at
            FROM runs
            LEFT JOIN users as ver ON runs.verifier = ver.id
            JOIN users as sub ON runs.submitter = sub.id
            WHERE game = $1 AND category = $2 AND status >= 1
            ORDER BY score DESC LIMIT 51"#,
        game.id.get(),
        category.id.get(),
    )
    .fetch_all(&state.postgres)
    .await?;
    Ok(crate::build_resolved_run_refs!(records, game, category))
}

async fn get_speedrun<'a>(
    state: &AppState,
    game: &'a Game,
    category: &'a Category,
) -> Result<Vec<ResolvedRunRef<'a>>, Error> {
    let records = query!(
        r#"SELECT runs.id, runs.game, runs.category, runs.video,
            runs.description, runs.score, runs.time, runs.status,
            runs.created_at, runs.verified_at,
            ver.id as "ver_id?", sub.id as sub_id,
            ver.username as "ver_name?", sub.username as sub_name,
            ver.has_stylesheet as "ver_has_stylesheet?",
            sub.has_stylesheet as sub_has_stylesheet,
            ver.biography as "ver_bio?", sub.biography as sub_bio,
            ver.pfp_ext as ver_pfp_ext, sub.pfp_ext as sub_pfp_ext,
            ver.banner_ext as ver_banner_ext,
            sub.banner_ext as sub_banner_ext,
            ver.admin as "ver_admin?", sub.admin as sub_admin,
            ver.created_at as "ver_created_at?",
            sub.created_at as sub_created_at
            FROM runs
            LEFT JOIN users as ver ON runs.verifier = ver.id
            JOIN users as sub ON runs.submitter = sub.id
            WHERE game = $1 AND category = $2 AND status >= 1
            ORDER BY time ASC LIMIT 51"#,
        game.id.get(),
        category.id.get(),
    )
    .fetch_all(&state.postgres)
    .await?;
    Ok(crate::build_resolved_run_refs!(records, game, category))
}

#[macro_export]
macro_rules! build_resolved_run_refs {
    ($records:expr, $game:expr, $category:expr) => {
        {
            let mut data: Vec<ResolvedRunRef> = Vec::with_capacity($records.len());
            for rec in $records {
                data.push($crate::build_resolved_run_ref!(rec, $game, $category));
            }
        data
        }
    };
}

#[macro_export]
macro_rules! build_resolved_run_ref {
    ($rec:ident, $game:expr, $category:expr) => {
        ResolvedRunRef {
            id: Id::new($rec.id),
            game: $game,
            category: $category,
            submitter: User {
                id: $rec.sub_id.into(),
                username: $rec.sub_name,
                has_stylesheet: $rec.sub_has_stylesheet,
                biography: $rec.sub_bio,
                pfp_ext: $rec.sub_pfp_ext,
                banner_ext: $rec.sub_banner_ext,
                admin: $rec.sub_admin,
                created_at: $rec.sub_created_at
            },
            verifier: opt_user(
                $rec.ver_id.map(Into::into),
                $rec.ver_name,
                $rec.ver_has_stylesheet,
                $rec.ver_bio,
                $rec.ver_pfp_ext,
                $rec.ver_banner_ext,
                $rec.ver_admin,
                $rec.ver_created_at
            ),
            video: $rec.video,
            description: $rec.description,
            score: $rec.score,
            time: $rec.time,
            status: RunStatus::from($rec.status),
            created_at: $rec.created_at,
            verified_at: $rec.verified_at
        }
    };
}