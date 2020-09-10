#![warn(missing_copy_implementations)]
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_contrib;

#[database("v")]
pub struct VDbConn(diesel::PgConnection);

mod deletion_token;
mod schema;

pub mod handlers;
pub mod models;
