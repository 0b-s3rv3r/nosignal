// mod app;
// mod db;
// mod error;
// mod network;
// mod schema;
// mod test;
mod ui;
mod util;

// use app::{get_command_request, App};
use ui::ui;

fn main() {
    // App::mem_init().run(get_command_request().unwrap());
    ui().unwrap();
}
