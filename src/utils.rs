use anyhow::Result;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::{distributions::Alphanumeric, Rng};
use rand_core::OsRng;


/// 对给定明文密码生成 Argon2 哈希
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e.to_string()))? // Manually map error
        .to_string();
    Ok(hash)
}

/// 校验明文密码是否匹配哈希
pub fn verify_password(hash: &str, password: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| anyhow::anyhow!(e.to_string()))?; // Manually map error
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// 生成指定长度的随机密码
pub fn generate_random_password(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
