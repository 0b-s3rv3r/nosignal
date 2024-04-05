mod app;
mod db;
mod error;
mod network;
mod schema;
mod util;
// mod ui;
// use ui::ChatUi;

use app::{get_command_request, App};

fn main() {
    App::init().run(get_command_request());
    // ChatUi::init().unwrap().run();
}
