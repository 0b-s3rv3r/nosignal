mod app;
mod db;
mod error;
mod network;
mod schema;
mod tui;
mod util;

use app::{get_command_request, run};
use error::AppError;
use log::error;

#[tokio::main]
async fn main() {
    if let Err(err) = run(get_command_request()).await {
        match err {
            AppError::PdbError(err) => error!("{}", err),
            AppError::IoError(err) => error!("{}", err),
            AppError::AlreadyExistingId => println!("{}", err),
            AppError::DataNotFound => println!("{}", err),
            AppError::AuthFailure => println!("{}", err),
            AppError::ConnectionRefused => println!("{}", err),
            AppError::NotExistingId => println!("{}", err),
            AppError::InvalidArgument => println!("{}", err),
            AppError::InvalidCommand => println!("{}", err),
        }
        std::process::exit(1);
    }
    std::process::exit(0);
}
