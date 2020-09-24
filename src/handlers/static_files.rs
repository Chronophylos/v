use lazy_static::lazy_static;
use resource::{resource, resource_str, Resource};
use rocket::{http::ContentType, response::Content};

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
