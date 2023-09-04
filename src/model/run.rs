use std::{cmp::Ordering, sync::Arc};

use chrono::NaiveDateTime;
use sqlx::{postgres::PgRow, Row};

use crate::{
    id::{CategoryMarker, GameMarker, Id, RunMarker, UserMarker},
    util::opt_user,
    AppState, Error,
};

use super::{Category, Game, User};

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy, sqlx::Type,
)]
#[repr(i8)]
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
    pub category: Category,
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

#[derive(Clone, Copy)]
pub enum SortBy {
    Score,
    Time,
    SubmissionDate(DateSort),
}

#[derive(Clone, Copy)]
pub enum DateSort {
    Newest,
    Oldest,
}

pub struct ResolvedRunResult {
    resolveds: Vec<ResolvedRun>,
    has_next: bool,
}

impl ResolvedRunResult {
    pub fn has_next(&self) -> bool {
        self.has_next
    }
    pub fn resolveds(self) -> Vec<ResolvedRun> {
        self.resolveds
    }
}

impl ResolvedRun {
    pub async fn from_db(
        state: &AppState,
        run_id: Id<RunMarker>,
    ) -> Result<Option<ResolvedRun>, Error> {
        let Some(rec) = sqlx::query(
            r#"SELECT runs.id, runs.game, runs.category, runs.video,
                runs.description, runs.score, runs.time, runs.status,
                runs.created_at, runs.verified_at,
                ver.id as ver_id,
                ver.username as ver_name,
                ver.has_stylesheet as ver_has_stylesheet,
                ver.biography as ver_bio,
                ver.pfp_ext as ver_pfp_ext,
                ver.banner_ext as ver_banner_ext,
                ver.admin as ver_admin,
                ver.created_at as ver_created_at,
                sub.id as sub_id,
                sub.username as sub_name,
                sub.has_stylesheet as sub_has_stylesheet,
                sub.biography as sub_bio,
                sub.pfp_ext as sub_pfp_ext,
                sub.banner_ext as sub_banner_ext,
                sub.admin as sub_admin,
                sub.created_at as sub_created_at,
                cat.game as cat_game,
                cat.name as cat_name,
                cat.description as cat_description,
                cat.rules as cat_rules,
                cat.scoreboard as cat_scoreboard,
                game.id as game_id,
                game.name as game_name,
                game.description as game_description,
                game.slug as game_slug,
                game.url as game_url,
                game.has_stylesheet as game_has_stylesheet,
                game.banner_ext as game_banner_ext,
                game.cover_art_ext as game_cover_art_ext,
                game.default_category as game_default_category
                FROM runs
                LEFT JOIN users as ver ON runs.verifier = ver.id
                JOIN users as sub ON runs.submitter = sub.id
                JOIN games as game ON game.id = runs.game
                JOIN categories as cat ON cat.id = runs.category
                WHERE runs.id = $1"#,
        )
        .bind(run_id.get())
        .fetch_optional(&state.postgres)
        .await?
        else {
            return Ok(None);
        };
        let id: Id<GameMarker> = rec.try_get("game_id")?;
        let name: String = rec.try_get("game_name")?;
        let description: String = rec.try_get("game_description")?;
        let slug: String = rec.try_get("game_slug")?;
        let url: String = rec.try_get("game_url")?;
        let has_stylesheet: bool = rec.try_get("game_has_stylesheet")?;
        let banner_ext: Option<String> = rec.try_get("game_banner_ext")?;
        let cover_art_ext: Option<String> = rec.try_get("game_cover_art_ext")?;
        let default_category: Id<CategoryMarker> = rec.try_get("game_default_category")?;
        let constructed_game = Game {
            id,
            name,
            slug,
            url,
            default_category,
            description,
            has_stylesheet,
            banner_ext,
            cover_art_ext,
        };
        Ok(Some(Self::row_to_rcat(&rec, &Arc::new(constructed_game))?))
    }
    pub async fn fetch_leaderboard(
        state: &AppState,
        game: Arc<Game>,
        status: RunStatus,
        maybe_category: Option<Id<CategoryMarker>>,
        sort_by: SortBy,
        limit: usize,
        page: usize,
    ) -> Result<ResolvedRunResult, Error> {
        let s_limit: i32 = limit.try_into()?;
        let page: i32 = page.try_into()?;
        let mut query = sqlx::QueryBuilder::new(
            r#"SELECT runs.id, runs.game, runs.category, runs.video,
            runs.description, runs.score, runs.time, runs.status,
            runs.created_at, runs.verified_at,
            ver.id as ver_id,
            ver.username as ver_name,
            ver.has_stylesheet as ver_has_stylesheet,
            ver.biography as ver_bio,
            ver.pfp_ext as ver_pfp_ext,
            ver.banner_ext as ver_banner_ext,
            ver.admin as ver_admin,
            ver.created_at as ver_created_at,
            sub.id as sub_id,
            sub.username as sub_name,
            sub.has_stylesheet as sub_has_stylesheet,
            sub.biography as sub_bio,
            sub.pfp_ext as sub_pfp_ext,
            sub.banner_ext as sub_banner_ext,
            sub.admin as sub_admin,
            sub.created_at as sub_created_at,
            cat.game as cat_game,
            cat.name as cat_name,
            cat.description as cat_description,
            cat.rules as cat_rules,
            cat.scoreboard as cat_scoreboard
            FROM runs
            LEFT JOIN users as ver ON runs.verifier = ver.id
            JOIN users as sub ON runs.submitter = sub.id
            JOIN categories as cat ON runs.category = cat.id
            WHERE runs.game = "#,
        );
        query.push_bind(game.id.get());
        if let Some(category) = maybe_category {
            query.push(" AND category = ");
            query.push_bind(category);
        }
        query.push(" AND status = ");
        query.push_bind(status as i16);
        match sort_by {
            SortBy::Score => query.push(" ORDER BY score DESC "),
            SortBy::Time => query.push(" ORDER BY time ASC "),
            SortBy::SubmissionDate(DateSort::Newest) => query.push(" ORDER BY created_at DESC "),
            SortBy::SubmissionDate(DateSort::Oldest) => query.push(" ORDER BY created_at ASC "),
        };
        query.push(" LIMIT ");
        query.push_bind(s_limit + 1);
        query.push(" OFFSET ");
        query.push_bind(s_limit * page);
        let rows = query.build().fetch_all(&state.postgres).await?;
        let mut resolveds = Vec::with_capacity(rows.len());
        for row in rows {
            resolveds.push(Self::row_to_rcat(&row, &game)?);
        }
        let has_next = resolveds.len() > limit;
        resolveds.truncate(limit);
        Ok(ResolvedRunResult {
            resolveds,
            has_next,
        })
    }
    fn row_to_rcat(v: &PgRow, game: &Arc<Game>) -> Result<ResolvedRun, Error> {
        let id: Id<RunMarker> = v.try_get("id")?;
        let game_id: Id<GameMarker> = v.try_get("game")?;
        let category_id: Id<CategoryMarker> = v.try_get("category")?;
        let video: String = v.try_get("video")?;
        let description: String = v.try_get("description")?;
        let score: i64 = v.try_get("score")?;
        let time: i64 = v.try_get("time")?;
        let status_num: i16 = v.try_get("status")?;
        let created_at: NaiveDateTime = v.try_get("created_at")?;
        let verified_at: Option<NaiveDateTime> = v.try_get("verified_at")?;

        let status = RunStatus::from(status_num);
        if game_id != game.id {
            return Err(Error::RowDoesNotMatchInputGame);
        }

        let ver_id: Option<Id<UserMarker>> = v.try_get("ver_id")?;
        let ver_name: Option<String> = v.try_get("ver_name")?;
        let ver_has_stylesheet: Option<bool> = v.try_get("ver_has_stylesheet")?;
        let ver_bio: Option<String> = v.try_get("ver_bio")?;
        let ver_pfp_ext: Option<String> = v.try_get("ver_pfp_ext")?;
        let ver_banner_ext: Option<String> = v.try_get("ver_banner_ext")?;
        let ver_admin: Option<bool> = v.try_get("ver_admin")?;
        let ver_created_at: Option<NaiveDateTime> = v.try_get("ver_created_at")?;

        let sub_id: Id<UserMarker> = v.try_get("sub_id")?;
        let sub_name: String = v.try_get("sub_name")?;
        let sub_has_stylesheet: bool = v.try_get("sub_has_stylesheet")?;
        let sub_bio: String = v.try_get("sub_bio")?;
        let sub_pfp_ext: Option<String> = v.try_get("sub_pfp_ext")?;
        let sub_banner_ext: Option<String> = v.try_get("sub_banner_ext")?;
        let sub_admin: bool = v.try_get("sub_admin")?;
        let sub_created_at: NaiveDateTime = v.try_get("sub_created_at")?;

        let cat_game_id: Id<GameMarker> = v.try_get("cat_game")?;
        let cat_name: String = v.try_get("cat_name")?;
        let cat_description: String = v.try_get("cat_description")?;
        let cat_rules: String = v.try_get("cat_rules")?;
        let cat_scoreboard: bool = v.try_get("cat_scoreboard")?;

        let verifier = opt_user(
            ver_id,
            ver_name,
            ver_has_stylesheet,
            ver_bio,
            ver_pfp_ext,
            ver_banner_ext,
            ver_admin,
            ver_created_at,
        );
        let submitter = User {
            id: sub_id,
            username: sub_name,
            has_stylesheet: sub_has_stylesheet,
            biography: sub_bio,
            pfp_ext: sub_pfp_ext,
            banner_ext: sub_banner_ext,
            admin: sub_admin,
            created_at: sub_created_at,
        };
        let category = Category {
            id: category_id,
            game: cat_game_id,
            name: cat_name,
            description: cat_description,
            rules: cat_rules,
            scoreboard: cat_scoreboard,
        };
        let rr = ResolvedRun {
            id,
            game: game.clone(),
            category,
            submitter,
            verifier,
            video,
            description,
            score,
            time,
            status,
            created_at,
            verified_at,
        };
        Ok(rr)
    }
}
