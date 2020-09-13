#![warn(missing_copy_implementations)]
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_contrib;

mod config;
mod deletion_token;
mod imgur;
mod schema;

pub mod handlers;
pub mod models;

use crate::handlers::*;
use config::Config;
use lazy_static::lazy_static;
use log::error;
use rocket::{catchers, fairing::AdHoc, http::Header, routes, Rocket};
use rocket_contrib::helmet::SpaceHelmet;
use rocket_contrib::{serve::StaticFiles, templates::Template};

lazy_static! {
    static ref STATIC_HEADERS: Vec<Header<'static>> = vec![
        Header::new("X-Server-Version", env!("CARGO_PKG_VERSION")),
        Header::new("X-Server-Name", env!("CARGO_PKG_NAME")),
        Header::new("X-Server-Commit", env!("GIT_COMMIT_HASH")),
        Header::new("X-Server-Framework", "Rocket")
    ];
}

#[database("v")]
pub struct VDbConn(diesel::PgConnection);

pub fn rocket() -> Rocket {
    rocket::ignite()
        .register(catchers![not_found])
        .mount("/assets", StaticFiles::from("assets"))
        .mount(
            "/",
            routes![
                index::get,
                index::head,
                index::new,
                index::import,
                index::feelsdankman,
                index::favicon,
            ],
        )
        .mount(
            "/a",
            routes![
                album::get,
                album::head,
                album::new,
                album::import,
                album::patch
            ],
        )
        .attach(SpaceHelmet::default())
        .attach(VDbConn::fairing())
        .attach(AdHoc::on_response("Server Headers", |_req, resp| {
            for header in STATIC_HEADERS.clone() {
                resp.set_header(header);
            }
            resp.remove_header("Server");
        }))
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("V Config", |rocket| {
            match Config::load("./config.toml") {
                Ok(c) => Ok(rocket.manage(c)),
                Err(err) => {
                    error!("Could not load config: {}", err);
                    Err(rocket)
                }
            }
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::Client;

    #[test]
    fn launch() {
        Client::new(rocket()).expect("valid rocket instance");
    }
}
