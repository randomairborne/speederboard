use axum::{
    extract::{Path, State},
    response::Html,
};
use tera::Context;

use crate::{
    id::{CategoryMarker, GameMarker, Id},
    model::{Category, Game, ResolvedRun, RunStatus, User},
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
    runs: Vec<ResolvedRun<'a>>,
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
    let runs: Vec<ResolvedRun> = if category.scoreboard {
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
    Ok(Html(state.tera.render("game.jinja", &ctx)?))
}

async fn get_scoreboard<'a>(
    state: &AppState,
    game: &'a Game,
    category: &'a Category,
) -> Result<Vec<ResolvedRun<'a>>, Error> {
    let records = query!(
        r#"SELECT runs.id, runs.game, runs.category, runs.video,
            runs.description, runs.score, runs.time, runs.status,
            ver.id as "ver_id?", sub.id as sub_id,
            ver.username as "ver_name?", sub.username as sub_name,
            ver.has_stylesheet as "ver_has_stylesheet?",
            sub.has_stylesheet as sub_has_stylesheet,
            ver.biography as "ver_bio?", sub.biography as sub_bio,
            ver.pfp_ext as ver_pfp_ext, sub.pfp_ext as sub_pfp_ext,
            ver.banner_ext as ver_banner_ext,
            sub.banner_ext as sub_banner_ext,
            ver.admin as "ver_admin?", sub.admin as sub_admin
            FROM runs
            LEFT JOIN users as ver ON runs.verifier = ver.id
            JOIN users as sub ON runs.submitter = sub.id
            WHERE game = $1 AND category = $2
            ORDER BY score DESC LIMIT 51"#,
        game.id.get(),
        category.id.get(),
    )
    .fetch_all(&state.postgres)
    .await?;
    let mut data: Vec<ResolvedRun> = Vec::with_capacity(records.len());
    for rec in records {
        data.push(ResolvedRun {
            id: Id::new(rec.id),
            game,
            category,
            submitter: User {
                id: rec.sub_id.into(),
                username: rec.sub_name,
                has_stylesheet: rec.sub_has_stylesheet,
                biography: rec.sub_bio,
                pfp_ext: rec.sub_pfp_ext,
                banner_ext: rec.sub_banner_ext,
                admin: rec.sub_admin,
            },
            verifier: opt_user(
                rec.ver_id.map(Into::into),
                rec.ver_name,
                rec.ver_has_stylesheet,
                rec.ver_bio,
                rec.ver_pfp_ext,
                rec.ver_banner_ext,
                rec.ver_admin,
            ),
            video: rec.video,
            description: rec.description,
            score: rec.score,
            time: rec.time,
            status: RunStatus::from(rec.status),
        });
    }
    Ok(data)
}

/// SO MUCH DUPLICATED CODE AHHHH
/// sqlx is pain, the types are technically different
async fn get_speedrun<'a>(
    state: &AppState,
    game: &'a Game,
    category: &'a Category,
) -> Result<Vec<ResolvedRun<'a>>, Error> {
    let records = query!(
        r#"SELECT runs.id, runs.game, runs.category, runs.video,
            runs.description, runs.score, runs.time, runs.status,
            ver.id as "ver_id?", sub.id as sub_id,
            ver.username as "ver_name?", sub.username as sub_name,
            ver.has_stylesheet as "ver_has_stylesheet?",
            sub.has_stylesheet as sub_has_stylesheet,
            ver.biography as "ver_bio?", sub.biography as sub_bio,
            ver.pfp_ext as ver_pfp_ext, sub.pfp_ext as sub_pfp_ext,
            ver.banner_ext as ver_banner_ext,
            sub.banner_ext as sub_banner_ext,
            ver.admin as "ver_admin?", sub.admin as sub_admin
            FROM runs
            LEFT JOIN users as ver ON runs.verifier = ver.id
            JOIN users as sub ON runs.submitter = sub.id
            WHERE game = $1 AND category = $2
            ORDER BY time ASC LIMIT 51"#,
        game.id.get(),
        category.id.get(),
    )
    .fetch_all(&state.postgres)
    .await?;
    let mut data: Vec<ResolvedRun> = Vec::with_capacity(records.len());
    for rec in records {
        data.push(ResolvedRun {
            id: Id::new(rec.id),
            game,
            category,
            submitter: User {
                id: rec.sub_id.into(),
                username: rec.sub_name,
                has_stylesheet: rec.sub_has_stylesheet,
                biography: rec.sub_bio,
                pfp_ext: rec.sub_pfp_ext,
                banner_ext: rec.sub_banner_ext,
                admin: rec.sub_admin,
            },
            verifier: opt_user(
                rec.ver_id.map(Into::into),
                rec.ver_name,
                rec.ver_has_stylesheet,
                rec.ver_bio,
                rec.ver_pfp_ext,
                rec.ver_banner_ext,
                rec.ver_admin,
            ),
            video: rec.video,
            description: rec.description,
            score: rec.score,
            time: rec.time,
            status: RunStatus::from(rec.status),
        });
    }
    Ok(data)
}
