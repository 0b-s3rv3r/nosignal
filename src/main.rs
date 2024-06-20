// mod app;
// mod db;
// mod error;
// mod network;
// mod schema;
// mod test;
mod ui;
// mod util;

use std::io;

use ui::App;

fn main() -> io::Result<()> {
    App::new().run()?;
    Ok(())
}
