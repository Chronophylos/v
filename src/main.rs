#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

use rocket::{catchers, routes};
use rocket_contrib::{serve::StaticFiles, templates::Template};
use v::{handlers::*, VDbConn};

pub fn main() {
    env_logger::init();

    rocket::ignite()
        .attach(VDbConn::fairing())
        .attach(Template::fairing())
        .register(catchers![not_found])
        .mount("/assets", StaticFiles::from("assets"))
        .mount("/", routes![index::get, index::head])
        .mount(
            "/a",
            routes![
                album::index,
                album::new,
                album::get,
                album::head,
                album::post,
                album::patch
            ],
        )
        .launch();
}
