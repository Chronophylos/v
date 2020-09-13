use std::collections::HashMap;

use log::trace;
use rocket::{http::Status, response::status::Custom};
use rocket_contrib::templates::Template;

#[get("/")]
pub fn get() -> Template {
    trace!("handling GET /");

    let context: HashMap<String, ()> = HashMap::new();
    Template::render("index", &context)
}

#[head("/")]
pub fn head() {
    trace!("handling HEAD /");
}

#[get("/new")]
pub fn new() -> Template {
    Template::render("new", ())
}

#[get("/import")]
pub fn import() -> Template {
    Template::render("import", ())
}

#[get("/FeelsDankMan")]
pub fn feelsdankman() -> Custom<&'static str> {
    Custom(Status::ImATeapot, "FeelsDankMan")
}
