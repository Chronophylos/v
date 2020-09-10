use std::collections::HashMap;

use rocket::Request;
use rocket_contrib::templates::Template;

pub mod album;
pub mod index;

#[catch(404)]
pub fn not_found(req: &Request) -> Template {
    let mut map = HashMap::new();
    map.insert("path", req.uri().path());
    Template::render("error/404", &map)
}
