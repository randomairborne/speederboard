use argon2::{PasswordHash, PasswordVerifier};
use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    id::{Id, UserMarker},
    util::AUTHTOKEN_COOKIE,
    AppState, Error,
};

/// pull it out of the DB with
/// `RETURNING id, username, has_stylesheet, pfp_ext, banner_ext, biography, admin`
#[derive(serde::Serialize, serde::Deserialize, Debug, Encode, Hash, PartialEq, Eq, Clone)]
pub struct User {
    pub id: Id<UserMarker>,
    pub username: String,
    pub has_stylesheet: bool,
    pub biography: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp_ext: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_ext: Option<String>,
    pub admin: bool,
    pub created_at: chrono::NaiveDateTime,
    pub flags: i64,
    pub language: Option<String>,
}

const DEFAULT_PFP: &str = "/static/pfp/default.png";

#[allow(dead_code)]
impl User {
    pub fn banner_dest_path(&self) -> String {
        format!("/customfiles/users/{}/banner.png", self.id)
    }

    pub fn pfp_dest_path(&self) -> String {
        format!("/customfiles/users/{}/pfp.png", self.id)
    }

    pub fn stylesheet_dest_path(&self) -> String {
        format!("/customfiles/users/{}/style.css", self.id)
    }

    pub fn stylesheet(&self) -> Option<String> {
        if self.has_stylesheet {
            Some(self.stylesheet_dest_path())
        } else {
            None
        }
    }

    pub fn pfp_path(&self) -> String {
        if self.pfp_ext.is_some() {
            self.pfp_dest_path()
        } else {
            DEFAULT_PFP.to_string()
        }
    }

    pub fn banner_path(&self) -> Option<String> {
        if self.has_stylesheet {
            Some(self.banner_dest_path())
        } else {
            None
        }
    }

    pub async fn from_db(state: &AppState, id: Id<UserMarker>) -> Result<User, Error> {
        let maybe_user: Option<User> =
            crate::util::get_redis_object(state, format!("user:{id}")).await?;
        if let Some(user) = maybe_user {
            return Ok(user);
        }
        let record = query!(
            "SELECT id, username, has_stylesheet, pfp_ext,
            banner_ext, biography, admin, created_at, flags,
            language
            FROM users WHERE id = $1",
            id.get()
        )
        .fetch_one(&state.postgres)
        .await?;
        let user = User {
            id: record.id.into(),
            username: record.username,
            has_stylesheet: record.has_stylesheet,
            pfp_ext: record.pfp_ext,
            banner_ext: record.banner_ext,
            biography: record.biography,
            admin: record.admin,
            created_at: record.created_at,
            flags: record.flags,
            language: record.language,
        };
        Ok(user)
    }

    pub async fn from_db_auth(
        state: &AppState,
        db: impl sqlx::PgExecutor<'_>,
        email: String,
        password: String,
    ) -> Result<Result<Self, ()>, Error> {
        let Some(record) = query!("SELECT * FROM users WHERE email = $1", email)
            .fetch_optional(db)
            .await?
        else {
            return Ok(Err(()));
        };
        let password_result = state
            .spawn_rayon(move |state| {
                let hash = PasswordHash::new(&record.password)?;
                state.argon.verify_password(password.as_ref(), &hash)
            })
            .await?;
        let user = User {
            id: record.id.into(),
            username: record.username,
            has_stylesheet: record.has_stylesheet,
            pfp_ext: record.pfp_ext,
            banner_ext: record.banner_ext,
            biography: record.biography,
            admin: record.admin,
            created_at: record.created_at,
            flags: record.flags,
            language: record.language,
        };
        if let Err(argon2::password_hash::Error::Password) = password_result {
            return Ok(Err(()));
        }
        // this looks a little weird! but we do this because if there's an error verifying
        // a password, we want to report it, but differently then if the password is *wrong*
        password_result?;
        Ok(Ok(user))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for User {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut redis = state.redis.get().await?;
        let jar = CookieJar::from_request_parts(parts, state).await?;
        let cookie = jar
            .get(AUTHTOKEN_COOKIE)
            .ok_or_else(|| Error::NeedsLogin(parts.uri.path().to_owned()))?;

        let maybe_user_id: Option<String> =
            redis.get(format!("token:user:{}", cookie.value())).await?;
        let user_id =
            maybe_user_id.ok_or_else(|| Error::NeedsLogin(parts.uri.path().to_owned()))?;

        let maybe_user: Option<String> = redis.get(format!("user:{user_id}")).await?;
        let user = maybe_user.ok_or(Error::TokenHasIdButIdIsUnkown)?;

        Ok(serde_json::from_str(&user)?)
    }
}

pub struct Admin(pub User);

impl AsRef<User> for Admin {
    fn as_ref(&self) -> &User {
        &self.0
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for Admin {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = User::from_request_parts(parts, state).await?;
        if user.admin {
            Ok(Admin(user))
        } else {
            Err(Error::InsufficientPermissions)
        }
    }
}

#[derive(Clone, Debug)]
pub struct UserUpdate {
    id: Id<UserMarker>,
    username: Option<String>,
    has_stylesheet: Option<bool>,
    biography: Option<String>,
    pfp_ext: MaybeNullUpdate<String>,
    banner_ext: MaybeNullUpdate<String>,
    admin: Option<bool>,
    flags: Option<i64>,
    language: MaybeNullUpdate<String>,
}

#[allow(dead_code)]
impl UserUpdate {
    pub fn new(id: Id<UserMarker>) -> Self {
        Self {
            id,
            username: None,
            has_stylesheet: None,
            biography: None,
            pfp_ext: MaybeNullUpdate::None,
            banner_ext: MaybeNullUpdate::None,
            admin: None,
            flags: None,
            language: MaybeNullUpdate::None,
        }
    }

    pub async fn execute(self, state: &AppState) -> Result<User, Error> {
        trace!(?self, "updating user with data");
        let new_db_user = query_as!(
            User,
            "UPDATE users SET
                username = COALESCE($2, username),
                has_stylesheet = COALESCE($3, has_stylesheet),
                biography = COALESCE($4, biography),
                pfp_ext = CASE WHEN $5 THEN NULL ELSE COALESCE($6, pfp_ext) END,
                banner_ext = CASE WHEN $7 THEN NULL ELSE COALESCE($8, banner_ext) END,
                language = CASE WHEN $9 THEN NULL ELSE COALESCE($10, language) END,
                admin = COALESCE($11, admin),
                flags = $12
            WHERE id = $1
            RETURNING id, username, has_stylesheet, flags,
            pfp_ext, banner_ext, biography, admin, created_at, language",
            self.id.get(),
            self.username,
            self.has_stylesheet,
            self.biography,
            self.pfp_ext.is_null(),
            self.pfp_ext.into_option(),
            self.banner_ext.is_null(),
            self.banner_ext.into_option(),
            self.language.is_null(),
            self.language.into_option(),
            self.admin,
            self.flags
        )
        .fetch_one(&state.postgres)
        .await?;
        trace!(?new_db_user, "updated user with data, adding to redis");
        state
            .redis
            .get()
            .await?
            .set_ex(
                format!("user:{}", self.id.get()),
                serde_json::to_string(&new_db_user)?,
                86_400,
            )
            .await?;
        Ok(new_db_user)
    }

    pub fn username(self, username: String) -> Self {
        Self {
            username: Some(username),
            ..self
        }
    }

    pub fn has_stylesheet(self, has_stylesheet: bool) -> Self {
        Self {
            has_stylesheet: Some(has_stylesheet),
            ..self
        }
    }

    pub fn biography(self, biography: String) -> Self {
        Self {
            biography: Some(biography),
            ..self
        }
    }

    pub fn pfp_ext(self, pfp_ext: Option<String>) -> Self {
        Self {
            pfp_ext: pfp_ext.into(),
            ..self
        }
    }

    pub fn banner_ext(self, banner_ext: Option<String>) -> Self {
        Self {
            banner_ext: banner_ext.into(),
            ..self
        }
    }

    pub fn admin(self, is_admin: bool) -> Self {
        Self {
            admin: Some(is_admin),
            ..self
        }
    }

    pub fn language(self, language: Option<String>) -> Self {
        Self {
            language: language.into(),
            ..self
        }
    }
}

#[derive(Clone, Debug)]
pub enum MaybeNullUpdate<T: Clone> {
    Null,
    None,
    Some(T),
}

impl<T: Clone> MaybeNullUpdate<T> {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn into_option(self) -> Option<T> {
        self.into()
    }
}

impl<T: Clone> From<Option<T>> for MaybeNullUpdate<T> {
    fn from(value: Option<T>) -> Self {
        if let Some(v) = value {
            Self::Some(v)
        } else {
            Self::Null
        }
    }
}

impl<T: Clone> From<MaybeNullUpdate<T>> for Option<T> {
    fn from(value: MaybeNullUpdate<T>) -> Option<T> {
        if let MaybeNullUpdate::Some(v) = value {
            Some(v)
        } else {
            None
        }
    }
}
