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
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
    Debug,
    Hash,
    PartialEq,
    Eq,
    Clone,
    Copy,
    sqlx::Type,
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Encode, Hash, PartialEq, Eq, Clone)]
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
    pub flags: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
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
    pub flags: i64,
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

struct ResolvedRunRequestMultiple {
    game: Arc<Game>,
    status: RunStatus,
    maybe_category: Option<Id<CategoryMarker>>,
    sort_by: SortBy,
    limit: usize,
    page: usize,
}
enum ResolvedRunRequest {
    Single(Id<RunMarker>),
    Multiple(ResolvedRunRequestMultiple),
}

impl ResolvedRun {
    pub async fn from_db(
        state: &AppState,
        run_id: Id<RunMarker>,
    ) -> Result<Option<ResolvedRun>, Error> {
        let resolveds = Self::run_fetcher(state, ResolvedRunRequest::Single(run_id)).await?;
        if resolveds.len() > 1 {
            return Err(Error::TooManyRows(1, resolveds.len()));
        }
        Ok(resolveds.into_iter().next())
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
        let request = ResolvedRunRequest::Multiple(ResolvedRunRequestMultiple {
            game,
            status,
            maybe_category,
            sort_by,
            limit,
            page,
        });
        let mut resolveds = Self::run_fetcher(state, request).await?;
        let has_next = resolveds.len() > limit;
        resolveds.truncate(limit);
        Ok(ResolvedRunResult {
            resolveds,
            has_next,
        })
    }

    async fn run_fetcher(
        state: &AppState,
        request: ResolvedRunRequest,
    ) -> Result<Vec<ResolvedRun>, Error> {
        let mut query = sqlx::QueryBuilder::new(
            r#"SELECT runs.id, runs.game, runs.category, runs.video,
            runs.description, runs.score, runs.time, runs.status,
            runs.created_at, runs.verified_at, runs.flags,
            verifier.id, verifier.username, verifier.has_stylesheet,
            verifier.biography, verifier.pfp_ext, verifier.banner_ext,
            verifier.admin, verifier.created_at, verifier.flags,
            submitter.id, submitter.username, submitter.has_stylesheet,
            submitter.biography, submitter.pfp_ext, submitter.banner_ext,
            submitter.admin, submitter.created_at, submitter.flags,
            category.game, category.name, category.description,
            category.rules, category.scoreboard, category.flags "#,
        );
        // getting a single run requires us to get game data as well
        if let ResolvedRunRequest::Single(_) = request {
            query.push(concat!(
                ',',
                "game.id, game.name, game.description, game.slug, game.url,",
                "game.has_stylesheet, game.banner_ext, game.cover_art_ext,",
                "game.default_category, game.flags ",
            ));
        }
        query.push(concat!(
            "FROM runs ",
            "LEFT JOIN users as verifier ON runs.verifier = verifier.id ",
            "JOIN users as submitter ON runs.submitter = submitter.id ",
            "JOIN categories as category ON runs.category = category.id ",
        ));
        if let ResolvedRunRequest::Single(id) = request {
            query.push(concat!(
                "JOIN games as game ON runs.game = game.id ",
                "WHERE runs.id = "
            ));
            query.push_bind(id);
        }
        if let ResolvedRunRequest::Multiple(multi_request) = &request {
            let s_limit: i64 = multi_request.limit.try_into()?;
            let page: i64 = multi_request.page.try_into()?;
            query.push("WHERE runs.game = ");
            query.push_bind(multi_request.game.id.get());
            if let Some(category) = multi_request.maybe_category {
                query.push(" AND category = ");
                query.push_bind(category);
            }
            query.push(" AND status = ");
            query.push_bind(multi_request.status as i16);
            match multi_request.sort_by {
                SortBy::Score => query.push(" ORDER BY score DESC "),
                SortBy::Time => query.push(" ORDER BY time ASC "),
                SortBy::SubmissionDate(DateSort::Newest) => {
                    query.push(" ORDER BY runs.created_at DESC ")
                }
                SortBy::SubmissionDate(DateSort::Oldest) => {
                    query.push(" ORDER BY runs.created_at ASC ")
                }
            };
            query.push(" LIMIT ");
            query.push_bind(s_limit + 1);
            query.push(" OFFSET ");
            query.push_bind(s_limit * page);
        }
        let rows = query.build().fetch_all(&state.postgres).await?;
        let mut resolveds = Vec::with_capacity(rows.len());
        let optional_game = match request {
            ResolvedRunRequest::Multiple(request) => Some(request.game),
            ResolvedRunRequest::Single(_) => None,
        };
        for row in rows {
            resolveds.push(Self::row_to_rcat(&row, optional_game.clone())?);
        }
        Ok(resolveds)
    }

    /// If `optional_game` is set, it will use the passed-in game. Otherwise,
    /// it will try to get it from the `game.*` fields
    fn row_to_rcat(row: &PgRow, optional_game: Option<Arc<Game>>) -> Result<ResolvedRun, Error> {
        let game = match optional_game {
            Some(v) => v,
            None => Self::get_game_from_row(row)?,
        };
        let id: Id<RunMarker> = row.try_get(0)?;
        let game_id: Id<GameMarker> = row.try_get(1)?;
        let category_id: Id<CategoryMarker> = row.try_get(2)?;
        let video: String = row.try_get(3)?;
        let description: String = row.try_get(4)?;
        let score: i64 = row.try_get(5)?;
        let time: i64 = row.try_get(6)?;
        let status_num: i16 = row.try_get(7)?;
        let created_at: NaiveDateTime = row.try_get(8)?;
        let verified_at: Option<NaiveDateTime> = row.try_get(9)?;
        let flags: i64 = row.try_get(10)?;

        let status = RunStatus::from(status_num);
        if game_id != game.id {
            return Err(Error::RowDoesNotMatchInputGame);
        }

        let verifier_id: Option<Id<UserMarker>> = row.try_get(11)?;
        let verifier_name: Option<String> = row.try_get(12)?;
        let verifier_has_stylesheet: Option<bool> = row.try_get(13)?;
        let verifier_bio: Option<String> = row.try_get(14)?;
        let verifier_pfp_ext: Option<String> = row.try_get(15)?;
        let verifier_banner_ext: Option<String> = row.try_get(16)?;
        let verifier_admin: Option<bool> = row.try_get(17)?;
        let verifier_created_at: Option<NaiveDateTime> = row.try_get(18)?;
        let verifier_flags: Option<i64> = row.try_get(19)?;

        let submitter_id: Id<UserMarker> = row.try_get(20)?;
        let submitter_name: String = row.try_get(21)?;
        let submitter_has_stylesheet: bool = row.try_get(22)?;
        let submitter_bio: String = row.try_get(23)?;
        let submitter_pfp_ext: Option<String> = row.try_get(24)?;
        let submitter_banner_ext: Option<String> = row.try_get(25)?;
        let submitter_admin: bool = row.try_get(26)?;
        let submitter_created_at: NaiveDateTime = row.try_get(27)?;
        let submitter_flags: i64 = row.try_get(28)?;

        let category_game_id: Id<GameMarker> = row.try_get(29)?;
        let category_name: String = row.try_get(30)?;
        let category_description: String = row.try_get(31)?;
        let category_rules: String = row.try_get(32)?;
        let category_scoreboard: bool = row.try_get(33)?;
        let category_flags: i64 = row.try_get(34)?;

        let verifier = opt_user(
            verifier_id,
            verifier_name,
            verifier_has_stylesheet,
            verifier_bio,
            verifier_pfp_ext,
            verifier_banner_ext,
            verifier_admin,
            verifier_created_at,
            verifier_flags
        );
        let submitter = User {
            id: submitter_id,
            username: submitter_name,
            has_stylesheet: submitter_has_stylesheet,
            biography: submitter_bio,
            pfp_ext: submitter_pfp_ext,
            banner_ext: submitter_banner_ext,
            admin: submitter_admin,
            created_at: submitter_created_at,
            flags: submitter_flags
        };
        let category = Category {
            id: category_id,
            game: category_game_id,
            name: category_name,
            description: category_description,
            rules: category_rules,
            scoreboard: category_scoreboard,
            flags: category_flags
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
            flags
        };
        Ok(rr)
    }

    fn get_game_from_row(row: &PgRow) -> Result<Arc<Game>, Error> {
        let id: Id<GameMarker> = row.try_get(35)?;
        let name: String = row.try_get(36)?;
        let description: String = row.try_get(37)?;
        let slug: String = row.try_get(38)?;
        let url: String = row.try_get(39)?;
        let has_stylesheet: bool = row.try_get(40)?;
        let banner_ext: Option<String> = row.try_get(41)?;
        let cover_art_ext: Option<String> = row.try_get(42)?;
        let default_category: Id<CategoryMarker> = row.try_get(43)?;
        let flags: i64 = row.try_get(44)?;
        Ok(Arc::new(Game {
            id,
            name,
            slug,
            url,
            default_category,
            description,
            has_stylesheet,
            banner_ext,
            cover_art_ext,
            flags
        }))
    }
}
