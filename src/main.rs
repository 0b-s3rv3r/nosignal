mod app;
mod db;
mod error;
mod network;
mod schema;
mod ui;

use app::{get_command_request, App};

fn main() {
    App::run(get_command_request());
}
