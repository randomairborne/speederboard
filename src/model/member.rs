use redis::AsyncCommands;

use super::{Permissions, User};
use crate::{
    id::{GameMarker, Id, UserMarker},
    AppState, Error,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
pub struct Member {
    pub perms: Permissions,
    #[serde(flatten)]
    pub user: User,
}

impl Member {
    pub async fn from_db(
        state: &AppState,
        user: Id<UserMarker>,
        game: Id<GameMarker>,
    ) -> Result<Option<Self>, Error> {
        let maybe_user: Option<User> =
            crate::util::get_redis_object(state, format!("user:{user}")).await?;
        let maybe_permissions: Option<i64> = state
            .redis
            .get()
            .await?
            .get(format!("permissions:{game}:{user}"))
            .await?;
        if let Some(user) = maybe_user {
            if let Some(perms) = maybe_permissions {
                return Ok(Some(Member {
                    perms: Permissions::new(perms),
                    user,
                }));
            }
        }
        let Some(member) = query!(
            "SELECT u.id, u.username, u.has_stylesheet,
            u.pfp, u.banner, u.biography, u.admin,
            u.created_at, u.flags, u.language, p.permissions
            FROM users as u LEFT JOIN permissions as p
            ON p.user_id = $1 AND p.game_id = $2 AND u.id = $1",
            user.get(),
            game.get()
        )
        .fetch_optional(&state.postgres)
        .await?
        else {
            return Ok(None);
        };
        let user = User {
            id: member.id.into(),
            username: member.username,
            has_stylesheet: member.has_stylesheet,
            biography: member.biography,
            pfp: member.pfp,
            banner: member.banner,
            admin: member.admin,
            created_at: member.created_at,
            flags: member.flags,
            language: member.language,
        };
        let perms = if user.admin {
            Permissions::ADMINISTRATOR
        } else {
            Permissions::new_opt(member.permissions)
        };
        state
            .redis
            .get()
            .await?
            .set_ex(format!("permissions:{game}:{}", user.id), perms.get(), 600)
            .await?;
        state
            .redis
            .get()
            .await?
            .set_ex(
                format!("user:{}", user.id),
                serde_json::to_string(&user)?,
                600,
            )
            .await?;
        Ok(Some(Member { perms, user }))
    }
}
