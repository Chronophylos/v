#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

use log::info;

pub fn main() {
    env_logger::init();

    info!(
        "Starting {} version {} ({})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("GIT_COMMIT_HASH")
    );

    v::rocket().launch();
}
