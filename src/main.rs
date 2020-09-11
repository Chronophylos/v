#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

use log::info;

pub fn main() {
    env_logger::init();

    info!("Starting v");

    v::rocket().launch();
}
