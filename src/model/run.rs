use std::{cmp::Ordering, sync::Arc};

use chrono::NaiveDateTime;
use sqlx::{postgres::PgRow, Row};

use crate::{
    id::{CategoryMarker, GameMarker, Id, RunMarker, UserMarker},
    util::opt_user,
    AppState, Error,
};

use super::{Category, Game, MiniCategory, User};

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy, sqlx::Type,
)]
pub enum RunStatus {
    Verified = 1,
    Rejected = -1,
    Pending = 0,
}

impl From<i64> for RunStatus {
    fn from(value: i64) -> Self {
        match value.cmp(&0) {
            Ordering::Equal => Self::Pending,
            Ordering::Less => Self::Rejected,
            Ordering::Greater => Self::Verified,
        }
    }
}

impl From<i32> for RunStatus {
    fn from(value: i32) -> Self {
        i64::from(value).into()
    }
}

impl From<i16> for RunStatus {
    fn from(value: i16) -> Self {
        i64::from(value).into()
    }
}

impl From<i8> for RunStatus {
    fn from(value: i8) -> Self {
        i64::from(value).into()
    }
}

impl From<RunStatus> for i8 {
    fn from(value: RunStatus) -> Self {
        match value {
            RunStatus::Pending => 0,
            RunStatus::Rejected => -1,
            RunStatus::Verified => 1,
        }
    }
}

impl From<RunStatus> for i16 {
    fn from(value: RunStatus) -> Self {
        i16::from(i8::from(value))
    }
}

impl From<RunStatus> for i32 {
    fn from(value: RunStatus) -> Self {
        i32::from(i8::from(value))
    }
}

impl From<RunStatus> for i64 {
    fn from(value: RunStatus) -> Self {
        i64::from(i8::from(value))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
pub struct Run {
    pub id: Id<RunMarker>,
    pub game: Id<GameMarker>,
    pub category: Id<CategoryMarker>,
    pub submitter: Id<UserMarker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<Id<UserMarker>>,
    pub video: String,
    pub description: String,
    pub score: i64,
    pub time: i64,
    pub status: RunStatus,
    pub created_at: NaiveDateTime,
    pub verified_at: Option<NaiveDateTime>,
}

#[derive(serde::Serialize, Debug, PartialEq, Clone)]
pub struct ResolvedRun {
    pub id: Id<RunMarker>,
    pub game: Arc<Game>,
    pub category: Arc<Category>,
    pub submitter: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<User>,
    pub video: String,
    pub description: String,
    pub score: i64,
    pub time: i64,
    pub status: RunStatus,
    pub created_at: NaiveDateTime,
    pub verified_at: Option<NaiveDateTime>,
}

impl ResolvedRun {
    pub async fn from_db(
        state: &AppState,
        run_id: Id<RunMarker>,
    ) -> Result<Option<ResolvedRun>, Error> {
        let Some(rec) = query!(
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
                ver.created_at as "ver_created_at?",
                sub.created_at as sub_created_at,
                ver.admin as "ver_admin?", sub.admin as sub_admin,
                cat.id as cat_id, cat.game as cat_game,
                cat.name as cat_name, cat.description as cat_description,
                cat.scoreboard as cat_scoreboard,
                cat.rules as cat_rules,
                game.name as game_name,
                game.description as game_description,
                game.slug as game_slug, game.url as game_url,
                game.has_stylesheet as game_has_stylesheet,
                game.banner_ext as game_banner_ext, game.id as game_id,
                game.cover_art_ext as game_cover_art_ext,
                game.default_category as game_default_category
                FROM runs
                LEFT JOIN users as ver ON runs.verifier = ver.id
                JOIN users as sub ON runs.submitter = sub.id
                JOIN games as game ON game.id = runs.game
                JOIN categories as cat ON cat.id = runs.category
                WHERE runs.id = $1"#,
            run_id.get()
        )
        .fetch_optional(&state.postgres)
        .await?
        else {
            return Ok(None);
        };
        Ok(Some(ResolvedRun {
            id: Id::new(rec.id),
            game: Arc::new(Game {
                id: rec.game_id.into(),
                name: rec.game_name,
                slug: rec.game_slug,
                url: rec.game_url,
                default_category: rec.game_default_category.into(),
                description: rec.game_description,
                has_stylesheet: rec.game_has_stylesheet,
                banner_ext: rec.game_banner_ext,
                cover_art_ext: rec.game_cover_art_ext,
            }),
            category: Arc::new(Category {
                id: rec.cat_id.into(),
                game: rec.cat_game.into(),
                name: rec.cat_name,
                description: rec.cat_description,
                rules: rec.cat_rules,
                scoreboard: rec.cat_scoreboard,
            }),
            submitter: User {
                id: rec.sub_id.into(),
                username: rec.sub_name,
                has_stylesheet: rec.sub_has_stylesheet,
                biography: rec.sub_bio,
                pfp_ext: rec.sub_pfp_ext,
                banner_ext: rec.sub_banner_ext,
                admin: rec.sub_admin,
                created_at: rec.sub_created_at,
            },
            verifier: opt_user(
                rec.ver_id.map(Into::into),
                rec.ver_name,
                rec.ver_has_stylesheet,
                rec.ver_bio,
                rec.ver_pfp_ext,
                rec.ver_banner_ext,
                rec.ver_admin,
                rec.ver_created_at,
            ),
            video: rec.video,
            description: rec.description,
            score: rec.score,
            time: rec.time,
            status: RunStatus::from(rec.status),
            created_at: rec.created_at,
            verified_at: rec.verified_at,
        }))
    }
}

#[derive(Clone)]
pub enum MaybeResolvedCategory {
    Resolved(Category),
    Id(Id<CategoryMarker>),
    NoConstraint,
}

#[derive(Clone, Copy)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl ResolvedRun {
    pub async fn fetch_leaderboard(
        state: &AppState,
        game: Game,
        triplet_category: MaybeResolvedCategory,
        order_direction: OrderDirection,
        limit: i32,
        status: RunStatus,
    ) -> Result<Vec<ResolvedRun>, Error> {
        let maybe_category = match triplet_category {
            MaybeResolvedCategory::Resolved(cat) => Some(cat),
            MaybeResolvedCategory::Id(id) => Some(
                Category::from_db(&state, id)
                    .await?
                    .ok_or(Error::NotFound)?,
            ),
            MaybeResolvedCategory::NoConstraint => None,
        };
        let game = Arc::new(game);
        let mut query = sqlx::QueryBuilder::new(
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
            WHERE game = "#,
        );
        query.push_bind(game.id.get());
        if let Some(category) = maybe_category {
            query.push(" AND category = ");
            query.push_bind(category.id);
        }
        query.push(" AND status = ");
        query.push_bind(status);
        query.push(" ORDER BY time ");
        query.push(match order_direction {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        });
        query.push(" LIMIT ");
        query.push_bind(limit);
        let data = query
            .build()
            .fetch_all(&state.postgres)
            .await?
            .into_iter()
            .map(|v| row_to_rcat(v, game.clone()))
            .collect();
        Ok(data)
    }
}

fn row_to_rcat(v: PgRow, game: Arc<Game>) -> Result<ResolvedRun, Error> {
    let id: i64 = v.get("id")?.;
    ResolvedRun {
        id: ,
        game: (),
        category: (),
        submitter: (),
        verifier: (),
        video: (),
        description: (),
        score: (),
        time: (),
        status: (),
        created_at: (),
        verified_at: (),
    }
}

#[macro_export]
macro_rules! build_resolved_run_refs {
    ($records:expr, $game:expr, $category:expr) => {{
        let mut data: Vec<ResolvedRunRef> = Vec::with_capacity($records.len());
        for rec in $records {
            data.push($crate::build_resolved_run_ref!(rec, $game, $category));
        }
        data
    }};
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
                created_at: $rec.sub_created_at,
            },
            verifier: opt_user(
                $rec.ver_id.map(Into::into),
                $rec.ver_name,
                $rec.ver_has_stylesheet,
                $rec.ver_bio,
                $rec.ver_pfp_ext,
                $rec.ver_banner_ext,
                $rec.ver_admin,
                $rec.ver_created_at,
            ),
            video: $rec.video,
            description: $rec.description,
            score: $rec.score,
            time: $rec.time,
            status: RunStatus::from($rec.status),
            created_at: $rec.created_at,
            verified_at: $rec.verified_at,
        }
    };
}
