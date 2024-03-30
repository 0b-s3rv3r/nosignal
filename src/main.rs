mod app;
mod db;
mod error;
mod network;
mod schema;
mod ui;

use app::App;

fn main() {
    App::run();
}
