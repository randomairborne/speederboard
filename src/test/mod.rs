// This module contains GLOBAL tests, not module-level ones, such as database migration and test tests
// it also contains test utilities

pub mod util;

use crate::Error;

#[sqlx::test(fixtures(path = "../fixtures", scripts("many_users", "many_games", "many_runs")))]
async fn many_fixtures(_db: sqlx::PgPool) -> Result<(), Error> {
    Ok(())
}

#[sqlx::test(fixtures(path = "../fixtures", scripts("add_user", "add_game", "add_run")))]
async fn one_fixture(_db: sqlx::PgPool) -> Result<(), Error> {
    Ok(())
}
