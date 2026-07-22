//! 登录认证模块 / Login Authentication Module

use crate::core::error::{AppError, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthData {
    pub username: String,
    pub password_hash: String,
}

pub struct AuthManager {
    data: RwLock<AuthData>,
    path: std::path::PathBuf,
    tokens: RwLock<HashSet<String>>,
}

impl AuthManager {
    pub fn new() -> Arc<Self> {
        let config_dir = crate::config::settings::AppState::config_dir();
        let _ = fs::create_dir_all(&config_dir);
        let path = config_dir.join("auth.json");
        let data = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str::<AuthData>(&c).ok())
                .unwrap_or_else(|| {
                    let default = AuthData {
                        username: "sr-mobileui".to_string(),
                        password_hash: hash_password("sr-mobileui", "admin"),
                    };
                    let _ = write_auth_file(&path, &default);
                    default
                })
        } else {
            let default = AuthData {
                username: "sr-mobileui".to_string(),
                password_hash: hash_password("sr-mobileui", "admin"),
            };
            let _ = write_auth_file(&path, &default);
            default
        };
        Arc::new(Self {
            data: RwLock::new(data),
            path,
            tokens: RwLock::new(HashSet::new()),
        })
    }

    pub fn login(&self, username: &str, password: &str) -> Option<String> {
        let data = self.data.read();
        if data.username == username && data.password_hash == hash_password(username, password) {
            let token = generate_token();
            self.tokens.write().insert(token.clone());
            Some(token)
        } else {
            None
        }
    }

    pub fn verify(&self, username: &str, password: &str) -> bool {
        let data = self.data.read();
        data.username == username && data.password_hash == hash_password(username, password)
    }

    pub fn check_token(&self, token: &str) -> bool {
        self.tokens.read().contains(token)
    }

    pub fn change_credentials(&self, new_username: &str, new_password: &str) -> Result<()> {
        let new_data = AuthData {
            username: new_username.to_string(),
            password_hash: hash_password(new_username, new_password),
        };
        let json = serde_json::to_string_pretty(&new_data)
            .map_err(|e| AppError::Other(format!("serialize auth.json: {}", e)))?;
        fs::write(&self.path, json)
            .map_err(|e| AppError::Other(format!("write auth.json: {}", e)))?;
        *self.data.write() = new_data;
        self.tokens.write().clear();
        Ok(())
    }

    pub fn username(&self) -> String {
        self.data.read().username.clone()
    }
}

fn write_auth_file(path: &std::path::Path, data: &AuthData) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    fs::write(path, json)
}

fn hash_password(username: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", username, password).as_bytes());
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

fn generate_token() -> String {
    let bytes: [u8; 32] = rand::random();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
