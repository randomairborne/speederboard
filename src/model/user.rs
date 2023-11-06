use argon2::{PasswordHash, PasswordVerifier};
use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    id::{Id, UserMarker},
    language::Language,
    util::AUTHTOKEN_COOKIE,
    AppState, Error,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Encode, Hash, PartialEq, Eq, Clone)]
pub struct User {
    pub id: Id<UserMarker>,
    pub username: String,
    pub stylesheet: bool,
    pub biography: String,
    pub pfp: bool,
    pub banner: bool,
    pub admin: bool,
    pub created_at: chrono::NaiveDateTime,
    pub flags: i64,
    pub language: Option<String>,
}

#[allow(dead_code)]
impl User {
    pub async fn from_db(state: &AppState, id: Id<UserMarker>) -> Result<User, Error> {
        let maybe_user: Option<User> =
            crate::util::get_redis_object(state, format!("user:{id}")).await?;
        if let Some(user) = maybe_user {
            return Ok(user);
        }
        let record = query!(
            "SELECT id, username, stylesheet, pfp,
            banner, biography, admin, created_at, flags,
            language
            FROM users WHERE id = $1",
            id.get()
        )
        .fetch_one(&state.postgres)
        .await?;
        let user = User {
            id: record.id.into(),
            username: record.username,
            stylesheet: record.stylesheet,
            pfp: record.pfp,
            banner: record.banner,
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
            .spawn_rayon(
                |state,
                 (phc_string, password)|
                 -> Result<Result<(), argon2::password_hash::Error>, Error> {
                    let hash = PasswordHash::new(&phc_string)?;
                    Ok(state.argon.verify_password(password.as_ref(), &hash))
                },
                (record.password, password),
            )
            .await??;
        let user = User {
            id: record.id.into(),
            username: record.username,
            stylesheet: record.stylesheet,
            pfp: record.pfp,
            banner: record.banner,
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

    pub fn check_admin(&self) -> Result<(), Error> {
        if self.admin {
            Ok(())
        } else {
            Err(Error::InsufficientPermissions)
        }
    }

    pub fn stylesheet_path(&self) -> String {
        format!("/users/{}/style.css", self.id)
    }

    pub fn pfp_path(&self, ext: &str) -> String {
        format!("/users/{}/pfp.{ext}", self.id)
    }

    pub fn banner_path(&self, ext: &str) -> String {
        format!("/users/{}/banner.{ext}", self.id)
    }

    pub fn stylesheet_url(&self, root: &str) -> String {
        root.to_owned() + &self.stylesheet_path()
    }

    pub fn pfp_url(&self, user_content: &str, static_root: &str, ext: &str) -> String {
        if self.pfp {
            user_content.to_owned() + &self.pfp_path(ext)
        } else {
            static_root.to_owned() + "/defaults/user/pfp.svg"
        }
    }

    pub fn banner_url(&self, user_content: &str, static_root: &str, ext: &str) -> String {
        if self.banner {
            user_content.to_owned() + &self.banner_path(ext)
        } else {
            static_root.to_owned() + "/defaults/user/banner.svg"
        }
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
    stylesheet: Option<bool>,
    biography: Option<String>,
    pfp: Option<bool>,
    banner: Option<bool>,
    admin: Option<bool>,
    flags: Option<i64>,
    language: MaybeNullUpdate<Language>,
}

#[allow(dead_code)]
impl UserUpdate {
    pub fn new(id: Id<UserMarker>) -> Self {
        Self {
            id,
            username: None,
            stylesheet: None,
            biography: None,
            pfp: None,
            banner: None,
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
                stylesheet = COALESCE($3, stylesheet),
                biography = COALESCE($4, biography),
                pfp = COALESCE($5, pfp),
                banner = COALESCE($6, banner),
                language = CASE WHEN $7 THEN NULL ELSE COALESCE($8, language) END,
                admin = COALESCE($9, admin),
                flags = COALESCE($10, flags)
            WHERE id = $1
            RETURNING id, username, stylesheet, flags,
            pfp, banner, biography, admin, created_at, language",
            self.id.get(),
            self.username,
            self.stylesheet,
            self.biography,
            self.pfp,
            self.banner,
            self.language.is_null(),
            self.language.into_option().map(Language::lang_code),
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

    pub fn stylesheet(self, stylesheet: bool) -> Self {
        Self {
            stylesheet: Some(stylesheet),
            ..self
        }
    }

    pub fn biography(self, biography: String) -> Self {
        Self {
            biography: Some(biography),
            ..self
        }
    }

    pub fn pfp(self, pfp: bool) -> Self {
        Self {
            pfp: Some(pfp),
            ..self
        }
    }

    pub fn banner(self, banner: bool) -> Self {
        Self {
            banner: Some(banner),
            ..self
        }
    }

    pub fn admin(self, is_admin: bool) -> Self {
        Self {
            admin: Some(is_admin),
            ..self
        }
    }

    pub fn language(self, language: Option<Language>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::InnerAppState;
    use crate::Error;
    use sqlx::{query, PgPool};

    #[sqlx::test(fixtures("add_user"))]
    async fn basic_user(db: PgPool) -> Result<(), Error> {
        let state = InnerAppState::test(db).await;
        let id = query!("SELECT id FROM users LIMIT 1")
            .fetch_one(&state.postgres)
            .await
            .unwrap();
        let user = User::from_db(&state, Id::new(id.id)).await.unwrap();
        assert!(!user.pfp);
        assert!(!user.banner);
        assert!(!user.stylesheet);
        assert_eq!(user.username, "test_user");
        assert_eq!(user.biography, "biography");
        assert!(!user.admin);
        Ok(())
    }
}
