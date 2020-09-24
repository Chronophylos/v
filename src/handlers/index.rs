use std::collections::HashMap;

use lazy_static::lazy_static;
use log::trace;
use resource::{resource, resource_str, Resource};
use rocket::{http::ContentType, http::Status, response::status::Custom, response::Content};
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

lazy_static! {
    static ref FAVICON: Resource<str> = resource_str!("assets/favicon.svg");
    static ref STYLES: Resource<str> = resource_str!("assets/styles.css");
    static ref BACKGROUND: Resource<[u8]> = resource!("assets/background.png");
}

#[get("/favicon.ico")]
pub fn favicon() -> Content<String> {
    get_resource_str(ContentType::SVG, &*FAVICON)
}

#[get("/styles.css")]
pub fn styles() -> Content<String> {
    get_resource_str(ContentType::CSS, &*STYLES)
}

#[get("/background.png")]
pub fn background() -> Content<Vec<u8>> {
    get_resource(ContentType::PNG, &*BACKGROUND)
}

fn get_resource_str(content_type: ContentType, resource: &Resource<str>) -> Content<String> {
    if resource.changed() {
        let mut resource = resource.clone();
        resource.reload_if_changed();
    }

    Content(content_type, resource.to_string())
}

fn get_resource(content_type: ContentType, resource: &Resource<[u8]>) -> Content<Vec<u8>> {
    if resource.changed() {
        let mut resource = resource.clone();
        resource.reload_if_changed();
    };

    Content(content_type, resource.to_vec())
}
