use argon2::{password_hash::Salt, Argon2, PasswordHasher};
use fern::Dispatch;
use humantime;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use uuid::Uuid;

pub fn get_unique_id() -> String {
    Uuid::new_v4().to_string()
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

pub fn create_env_dir(dir_name: &str) -> Result<PathBuf, std::io::Error> {
    let data_dir = match dirs::data_dir() {
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
        fs::create_dir_all(&dir_path)?;
    }

    Ok(dir_path)
}

pub fn setup_logger(log_path: &Path) -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
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
