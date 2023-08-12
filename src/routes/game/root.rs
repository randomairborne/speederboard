use axum::{
    extract::{Path, State},
    response::Html,
};
use tera::Context;

use crate::{
    id::{CategoryMarker, Id},
    model::{Category, Game, Run},
    template::BaseRenderInfo,
    AppState, Error,
};

#[derive(serde::Serialize)]
pub struct GetGameContext<'a> {
    #[serde(flatten)]
    core: BaseRenderInfo<'a>,
    categories: Vec<Category>,
    selected: Category,
    runs: Vec<Run>,
    game: Game,
}

pub async fn get(
    State(state): State<AppState>,
    Path((game_slug, category_slug)): Path<(String, Option<String>)>,
) -> Result<Html<String>, Error> {
    let game = Game::from_db_slug(&state, game_slug).await?;
    let categories = query_as!(
        Category,
        "SELECT * FROM categories WHERE game = $1",
        game.id.get()
    )
    .fetch_all(&state.postgres)
    .await?;
    let selected =
        get_selected(&categories, &category_slug, game.default_category).ok_or(Error::NotFound)?;
    let runs = query_as!(
        Run,
        "SELECT * FROM runs
        WHERE game = $1 AND category = $2
        ORDER BY data->$3 ASC",
        game.id.get()
    )
    .fetch_all(&state.postgres)
    .await?;
    let get_game_ctx = GetGameContext {
        core: state.base_context(),
        categories,
        selected,
        game,
        runs,
    };
    let ctx = Context::from_serialize(get_game_ctx)?;
    Ok(Html(state.tera.render("game.jinja", &ctx)?))
}

fn get_selected(
    categories: &[Category],
    category_slug: &Option<String>,
    default_category: Id<CategoryMarker>,
) -> Option<Category> {
    categories
        .into_iter()
        .filter(|v| {
            if let Some(slug) = category_slug {
                &v.slug == slug
            } else {
                v.id == default_category
            }
        })
        .nth(0)
        .cloned()
}
