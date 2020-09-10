use std::collections::HashMap;

use log::trace;
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
