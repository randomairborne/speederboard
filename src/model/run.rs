use std::cmp::Ordering;

use crate::{
    id::{CategoryMarker, GameMarker, Id, RunMarker, UserMarker},
    util::opt_user,
    AppState, Error,
};

use super::{Category, Game, User};

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy, sqlx::Type,
)]
pub enum RunStatus {
    Verified,
    Rejected,
    Pending,
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
}

#[derive(serde::Serialize, Debug, PartialEq, Clone)]
pub struct ResolvedRunRef<'a> {
    pub id: Id<RunMarker>,
    pub game: &'a Game,
    pub category: &'a Category,
    pub submitter: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<User>,
    pub video: String,
    pub description: String,
    pub score: i64,
    pub time: i64,
    pub status: RunStatus,
}

#[derive(serde::Serialize, Debug, PartialEq, Clone)]
pub struct ResolvedRun {
    pub id: Id<RunMarker>,
    pub game: Game,
    pub category: Category,
    pub submitter: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<User>,
    pub video: String,
    pub description: String,
    pub score: i64,
    pub time: i64,
    pub status: RunStatus,
}

impl ResolvedRun {
    pub async fn from_db(
        state: &AppState,
        run_id: Id<RunMarker>,
    ) -> Result<Option<ResolvedRun>, Error> {
        let Some(rec) = query!(
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
                ver.admin as "ver_admin?", sub.admin as sub_admin,
                cat.id as cat_id, cat.game as cat_game,
                cat.name as cat_name, cat.description as cat_description,
                cat.scoreboard as cat_scoreboard,
                cat.rules as cat_rules, game.name as game_name,
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
            game: Game {
                id: rec.game_id.into(),
                name: rec.game_name,
                slug: rec.game_slug,
                url: rec.game_url,
                default_category: rec.game_default_category.into(),
                description: rec.game_description,
                has_stylesheet: rec.game_has_stylesheet,
                banner_ext: rec.game_banner_ext,
                cover_art_ext: rec.game_cover_art_ext,
            },
            category: Category {
                id: rec.cat_id.into(),
                game: rec.cat_game.into(),
                name: rec.cat_name,
                description: rec.cat_description,
                rules: rec.cat_rules,
                scoreboard: rec.cat_scoreboard,
            },
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
        }))
    }
}
