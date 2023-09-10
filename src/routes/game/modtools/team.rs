use axum::{
    extract::{Path, State},
    response::Redirect,
};
use redis::AsyncCommands;

use crate::{
    id::{Id, UserMarker},
    model::{Game, Member, Permissions, User},
    template::BaseRenderInfo,
    util::{game_n_member, ValidatedForm},
    AppState, Error, HandlerResult,
};

#[derive(serde::Serialize, Debug, Clone)]
pub struct TeamPage {
    members: Vec<Member>,
    #[serde(flatten)]
    base: BaseRenderInfo,
}

#[derive(serde::Deserialize, garde::Validate, Debug, Clone)]
pub struct ModifyTeamMemberForm {
    #[garde(skip)]
    member: Id<UserMarker>,
    #[garde(skip)]
    #[serde(flatten)]
    permissions: Permissions,
}

pub async fn get(
    State(state): State<AppState>,
    Path(game_slug): Path<String>,
    user: User,
    base: BaseRenderInfo,
) -> HandlerResult {
    let game = Game::from_db_slug(&state, &game_slug).await?;
    let member = Member::from_db(&state, user.id, game.id)
        .await?
        .ok_or(Error::InsufficientPermissions)?;
    if !member.perms.contains(Permissions::ADMINISTRATOR) {
        return Err(Error::InsufficientPermissions);
    }
    let members = query!(
        "SELECT permissions.permissions,
        users.id, users.username, users.biography,
        users.admin, users.has_stylesheet, users.banner_ext,
        users.pfp_ext, users.flags, users.created_at
        FROM users
        JOIN permissions ON permissions.user_id = users.id
        WHERE permissions.permissions > 0
        AND permissions.game_id = $1",
        game.id.get()
    )
    .fetch_all(&state.postgres)
    .await?
    .into_iter()
    .map(|row| Member {
        perms: Permissions::new(row.permissions),
        user: User {
            id: Id::new(row.id),
            username: row.username,
            has_stylesheet: row.has_stylesheet,
            biography: row.biography,
            pfp_ext: row.pfp_ext,
            banner_ext: row.banner_ext,
            admin: row.admin,
            created_at: row.created_at,
            flags: row.flags,
        },
    })
    .collect();
    let ctx = TeamPage { members, base };
    state.render("game_team.jinja", ctx)
}

pub async fn post(
    State(state): State<AppState>,
    Path(game_slug): Path<String>,
    user: User,
    ValidatedForm(form): ValidatedForm<ModifyTeamMemberForm>,
) -> Result<Redirect, Error> {
    let (game, member) = game_n_member(&state, user, &game_slug).await?;
    if !member.perms.contains(Permissions::ADMINISTRATOR) {
        return Err(Error::InsufficientPermissions);
    }
    state
        .redis
        .get()
        .await?
        .set_ex(
            format!("permissions:{}:{}", game.id, member.user.id.get()),
            form.permissions.get(),
            600,
        )
        .await?;
    query!(
        "INSERT INTO permissions (user_id, game_id, permissions)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, game_id) DO UPDATE SET permissions = $3",
        form.member.get(),
        game.id.get(),
        form.permissions.get()
    )
    .execute(&state.postgres)
    .await?;
    Ok(state.redirect(format!("/game/{game_slug}/team")))
}
