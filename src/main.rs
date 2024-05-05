mod app;
mod db;
mod error;
mod network;
mod schema;
mod test;
mod ui;
mod util;
<<<<<<< Updated upstream
// use ui::ChatUi;
=======
>>>>>>> Stashed changes

use app::{get_command_request, App};

fn main() {
    App::mem_init().run(get_command_request().unwrap());
    // ChatUi::init().unwrap().run();
}
