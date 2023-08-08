use argon2::{PasswordHash, PasswordVerifier};
use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

use crate::{
    id::{Id, UserMarker},
    AppState, Error,
};

/// pull it out of the DB with
/// `RETURNING id, username, has_stylesheet, pfp_ext, banner_ext, biography`
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Id<UserMarker>,
    pub username: String,
    pub has_stylesheet: bool,
    pub biography: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp_ext: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_ext: Option<String>,
}

const DEFAULT_PFP: &str = "/static/pfp/default.png";

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
    pub async fn from_db(
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

pub const TOKEN_COOKIE: &str = "token";

#[axum::async_trait]
impl FromRequestParts<AppState> for User {
    type Rejection = Error;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut redis = state.redis.get().await?;
        let jar = CookieJar::from_request_parts(parts, state).await?;
        let cookie = jar.get(TOKEN_COOKIE).ok_or(Error::InvalidCookie)?;

        let maybe_user_id: Option<String> =
            redis.get(format!("token:user:{}", cookie.value())).await?;
        let user_id = maybe_user_id.ok_or(Error::InvalidCookie)?;

        let maybe_user: Option<String> = redis.get(format!("user:{user_id}")).await?;
        let user = maybe_user.ok_or(Error::TokenHasIdButIdIsUnkown)?;

        Ok(serde_json::from_str(&user)?)
    }
}
