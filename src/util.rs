use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use argon2::{password_hash::Salt, Argon2, PasswordHasher};
use uuid::Uuid;

pub fn get_unique_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn passwd_input() -> String {
    print!("password: ");
    io::stdout().flush().unwrap();

    let passwd = rpassword::read_password().unwrap();
    hash_passwd(&passwd);

    passwd
}

pub fn hash_passwd(passwd: &str) {
    Argon2::default()
        .hash_password(
            passwd.as_bytes(),
            Salt::from_b64("supersecretsalt").unwrap(),
        )
        .unwrap();
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
