use jwt::Token;

// 生成token
pub fn generate_token(
    user_uuid: &str,
    expiration: u64,
    secret: &[u8],
) -> Result<String, jwt::Error> {
    use aes_gcm::KeyInit;
    use hmac::Hmac;
    use jwt::SignWithKey;
    use std::collections::BTreeMap;

    let key: Hmac<sha2::Sha256> = Hmac::new_from_slice(secret).unwrap();

    let mut claims = BTreeMap::new();
    claims.insert("uuid", format!("{}", user_uuid));
    claims.insert(
        "exp",
        format!("{}", chrono::Utc::now().timestamp() + expiration as i64),
    );
    claims.insert("iat", format!("{}", chrono::Utc::now().timestamp()));

    let header = jwt::Header {
        algorithm: jwt::AlgorithmType::Hs256,
        ..Default::default()
    };

    Token::new(header, claims).sign_with_key(&key).map(|v| v.as_str().to_string())
}

// 校验token
pub fn verify_token(token_str: &str, secret: &[u8]) -> Result<String, anyhow::Error> {
    use aes_gcm::KeyInit;
    use hmac::Hmac;
    use jwt::VerifyWithKey;
    use sha2::Sha256;
    use std::collections::BTreeMap;

    let key: Hmac<Sha256> = Hmac::new_from_slice(secret)?;
    let claims: BTreeMap<String, String> = token_str.verify_with_key(&key)?;
    let exp_str = claims.get("exp");

    // 检查过期时间
    let exp = exp_str
        .ok_or_else(|| anyhow::anyhow!("token expired."))?
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("token expired."))?;
    let current_time = chrono::Utc::now().timestamp() as u64;
    if current_time > exp {
        return Err(anyhow::anyhow!("token expired."));
    }

    let uuid = claims.get("uuid");
    match uuid {
        Some(uuid) => return Ok(uuid.to_string()),
        None => Err(anyhow::anyhow!("token parse failed.")),
    }
}
