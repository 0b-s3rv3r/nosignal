use argon2::{password_hash::Salt, Argon2, PasswordHasher};
use chrono::{DateTime, Local, Utc};
use dirs::data_dir;
use fern::Dispatch;
use humantime::format_rfc3339_seconds;
use std::{
    fs::create_dir_all,
    io::{self, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};
use uuid::Uuid;

pub fn get_unique_id() -> String {
    format!("user{}", Uuid::new_v4())
}

pub fn passwd_input() -> String {
    print!("password: ");
    io::stdout().flush().unwrap();
    let passwd = rpassword::read_password().unwrap();
    hash_passwd(&passwd)
}

pub fn hash_passwd(passwd: &str) -> String {
    Argon2::default()
        .hash_password(
            passwd.as_bytes(),
            Salt::from_b64("c3VwZXJzZWNyZXRzYWx0").unwrap(),
        )
        .unwrap()
        .hash
        .unwrap()
        .to_string()
}

pub fn create_env_dir(dir_name: &str) -> Result<PathBuf, io::Error> {
    let data_dir = match data_dir() {
        Some(dir) => dir,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unable to determine data directory",
            ));
        }
    };

    let dir_path = data_dir.join(dir_name);

    if !dir_path.exists() {
        create_dir_all(&dir_path)?;
    }

    Ok(dir_path)
}

pub fn setup_logger(log_path: &Path) -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Error)
        .chain(std::io::stdout())
        .chain(fern::log_file(&log_path)?)
        .apply()?;

    Ok(())
}

pub fn systime_to_string(time: SystemTime) -> String {
    let local = Local::now();
    let offset = local.offset();
    let tz_time = DateTime::<Utc>::from(time) + *offset;
    tz_time.format("%Y-%m-%d %H:%M").to_string()
}
