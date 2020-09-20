#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

#[macro_use]
extern crate diesel_migrations;

use anyhow::Result;
use diesel::{Connection, PgConnection};
use log::info;
use rocket_contrib::databases::database_config;
use self_update::cargo_crate_version;

// provides embedded_migrations
embed_migrations!();

pub fn main() -> Result<()> {
    env_logger::init();

    // Update binary when running in release mode
    #[cfg(not(debug_assertions))]
    v::update()?;

    info!(
        "Starting {} version {} ({})",
        env!("CARGO_PKG_NAME"),
        cargo_crate_version!(),
        env!("GIT_COMMIT_HASH")
    );

    info!("Building rocket");
    let rocket = v::rocket();

    info!("Running database migrations");
    let db_config = database_config("v", rocket.config()).unwrap();
    let conn = PgConnection::establish(db_config.url)?;
    embedded_migrations::run(&conn)?;

    rocket.launch();

    Ok(())
}
