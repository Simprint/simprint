use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// 对密码进行加密
pub fn encryption_password(password: &str) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(password_hash) => Ok(password_hash.to_string()),
        Err(_) => Err(anyhow::anyhow!("密码异常")),
    }
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = if let Ok(parsed_hash) = PasswordHash::new(&password_hash) {
        parsed_hash
    } else {
        return false;
    };

    Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
}
