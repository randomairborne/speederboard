use crate::{
    id::{GameMarker, Id, UserMarker},
    AppState, Error,
};

use super::{Permissions, User};

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
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
        let member_opt = query!(
            "SELECT u.id, u.username, u.has_stylesheet,
            u.pfp_ext, u.banner_ext, u.biography, u.admin,
            u.created_at, p.permissions
            FROM users as u LEFT JOIN permissions as p
            ON p.user_id = $1 AND p.game_id = $2 AND u.id = $1",
            user.get(),
            game.get()
        )
        .fetch_optional(&state.postgres)
        .await?;
        let Some(member) = member_opt else {
            return Ok(None);
        };
        let user = User {
            id: member.id.into(),
            username: member.username,
            has_stylesheet: member.has_stylesheet,
            biography: member.biography,
            pfp_ext: member.pfp_ext,
            banner_ext: member.banner_ext,
            admin: member.admin,
            created_at: member.created_at,
        };
        let perms = if user.admin {
            Permissions::ADMINISTRATOR
        } else {
            Permissions::new_opt(member.permissions)
        };
        Ok(Some(Member { perms, user }))
    }
}
