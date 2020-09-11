#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

#[macro_use]
extern crate diesel_migrations;

use diesel::{Connection, PgConnection};
use log::info;
use rocket_contrib::databases::database_config;

// provides embedded_migrations
embed_migrations!();

pub fn main() {
    env_logger::init();

    info!(
        "Starting {} version {} ({})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_COMMIT_HASH")
    );

    let rocket = v::rocket();

    info!("Running database migrations");
    let db_config = database_config("v", rocket.config()).unwrap();
    let conn = PgConnection::establish(db_config.url).unwrap();
    embedded_migrations::run(&conn).unwrap();

    rocket.launch();
}
