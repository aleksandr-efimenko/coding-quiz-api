use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use crate::models::Claims;
use actix_web::{FromRequest, Error as ActixError, http};
use std::future::{Ready, ready};
use actix_web::dev::Payload;

const SECRET: &[u8] = b"supersecretkeydontshareThisIsJustATestKeyPleaseChangeItInProd"; // Should be in env in prod

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(password_hash)
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
}

pub fn create_token(username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET))
}

pub fn decode_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

// Extractor
pub struct JwtMiddleware {
    pub user_id: String, // Storing username in sub, so this is username effectively
}

impl FromRequest for JwtMiddleware {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get(http::header::AUTHORIZATION);
        
        if let Some(auth_val) = auth_header {
            if let Ok(auth_str) = auth_val.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    if let Ok(claims) = decode_token(token) {
                        return ready(Ok(JwtMiddleware { user_id: claims.sub }));
                    }
                }
            }
        }
        
        ready(Err(actix_web::error::ErrorUnauthorized("Invalid or missing token")))
    }
}
