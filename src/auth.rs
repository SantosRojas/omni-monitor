use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::models::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub user_id: i64,
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub iat: u64,
    pub exp: u64,
    pub iss: String,
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(stored: &str, candidate: &str) -> bool {
    let parsed = match PasswordHash::new(stored) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse stored password hash");
            return false;
        }
    };
    Argon2::default().verify_password(candidate.as_bytes(), &parsed).is_ok()
}

pub fn issue_token(
    user: &User,
    secret: &str,
    expiration_hours: u64,
    issuer: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now().timestamp() as u64;
    let claims = JwtClaims {
        user_id: user.id,
        username: user.username.clone(),
        full_name: user.full_name.clone(),
        email: user.email.clone(),
        role: user.role.clone(),
        iat: now,
        exp: now + (expiration_hours * 3600),
        iss: issuer.to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_token(
    token: &str,
    secret: &str,
    issuer: &str,
) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.set_issuer(&[issuer]);
    validation.validate_exp = true;
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    Ok(token_data.claims)
}
