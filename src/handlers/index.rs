use std::collections::HashMap;

use log::trace;
use rocket::response::Redirect;
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

#[get("/favicon.ico")]
pub fn favicon() -> Redirect {
    Redirect::to("/assets/favicon.svg")
}
