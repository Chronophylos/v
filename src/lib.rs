#![warn(missing_copy_implementations)]
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel_migrations;

mod deletion_token;
mod schema;

pub mod handlers;
pub mod models;

use std::{collections::HashSet, ops::Deref};

use crate::handlers::*;
use lazy_static::lazy_static;
use rocket::{catchers, fairing::AdHoc, http::Header, routes, Rocket};
use rocket_contrib::{serve::StaticFiles, templates::Template};

lazy_static! {
    static ref STATIC_HEADERS: Vec<Header<'static>> = vec![
        Header::new("X-Server-Version", env!("CARGO_PKG_VERSION")),
        Header::new("X-Server-Name", env!("CARGO_PKG_NAME")),
        Header::new("X-Server-Commit", env!("GIT_COMMIT_HASH")),
        Header::new("X-Server-Framework", "Rocket")
    ];
}

// provides embedded_migrations
embed_migrations!();

#[database("v")]
pub struct VDbConn(diesel::PgConnection);

#[derive(Debug)]
pub struct DomainAllowList(HashSet<String>);

impl Deref for DomainAllowList {
    type Target = HashSet<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn rocket() -> Rocket {
    rocket::ignite()
        .attach(VDbConn::fairing())
        .attach(AdHoc::on_response("Server Headers", |_req, resp| {
            for header in STATIC_HEADERS.clone() {
                resp.set_header(header);
            }
            resp.remove_header("Server");
        }))
        .attach(Template::fairing())
        .attach(AdHoc::on_attach("Domain Allow List Config", |rocket| {
            let set = rocket
                .config()
                .get_slice("domain_allow_list")
                .unwrap_or(&Vec::new())
                .iter()
                .map(|value| value.as_str().unwrap().to_string())
                .fold(HashSet::new(), |mut set, value| {
                    set.insert(value);
                    set
                });

            Ok(rocket.manage(DomainAllowList(set)))
        }))
        .register(catchers![not_found])
        .mount("/assets", StaticFiles::from("assets"))
        .mount("/", routes![index::get, index::head])
        .mount(
            "/a",
            routes![
                album::index,
                album::get,
                album::head,
                album::post,
                album::patch
            ],
        )
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
