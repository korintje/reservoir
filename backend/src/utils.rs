use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs, io};

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub web_url: String,
    pub db_path: String,
    pub admin_password_hash: String,
    pub secure_cookies: bool,
    pub basic_auth_enabled: bool,
    pub public_mode: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            web_url: "127.0.0.1:8080".into(),
            db_path: "./db/reservoir.db".into(),
            admin_password_hash: String::new(),
            secure_cookies: true,
            basic_auth_enabled: true,
            public_mode: "lan".into(),
        }
    }
}
impl Config {
    pub fn load() -> io::Result<Self> {
        let path = std::env::var("RESERVOIR_CONFIG").unwrap_or_else(|_| "reservoir.toml".into());
        toml::from_str(&fs::read_to_string(path)?).map_err(io::Error::other)
    }
}
pub fn hash(input: &str) -> String {
    format!("{:X}", Sha256::digest(input.as_bytes()))
}
#[cfg(test)]
pub fn hash_anyway(input: &Option<String>) -> String {
    hash(input.as_deref().unwrap_or(""))
}
pub fn secure_hash(input: &str) -> Result<String, String> {
    Argon2::default()
        .hash_password(input.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}
pub fn verify_password(input: &str, stored: Option<&str>) -> bool {
    match stored {
        None | Some("") => input.is_empty(),
        Some(value) if value.starts_with("$argon2") => {
            PasswordHash::new(value).ok().is_some_and(|parsed| {
                Argon2::default()
                    .verify_password(input.as_bytes(), &parsed)
                    .is_ok()
            })
        }
        Some(value) => hash(input) == value,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn passwords_support_legacy_and_salted_hashes() {
        let salted = secure_hash("correct").unwrap();
        assert!(verify_password("correct", Some(&salted)));
        assert!(!verify_password("wrong", Some(&salted)));
        assert!(verify_password("correct", Some(&hash("correct"))));
        assert!(verify_password("", None));
        assert!(!verify_password("wrong", None));
    }
}
