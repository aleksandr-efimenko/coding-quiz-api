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
use crate::state::AppState;
use crate::id::Id;
use actix_web::{web, FromRequest, Error as ActixError, http};
use std::future::{Ready, ready};
use actix_web::dev::Payload;
use sha2::{Sha256, Digest};
use rand::{distributions::Alphanumeric, Rng};
use std::pin::Pin;
use std::future::Future;

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

pub fn create_token(username: &str, developer_id: Id) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration,
        developer_id: developer_id.to_i64(),
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

pub fn generate_api_key() -> (String, String) {
    let key: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    
    (key, hash)
}

pub struct ApiKeyMiddleware {
    pub developer_id: Id,
    pub api_key_id: Id,
}

impl FromRequest for ApiKeyMiddleware {
    type Error = ActixError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let key_val = match req.headers().get("X-API-Key") {
                Some(k) => k,
                None => return Err(actix_web::error::ErrorUnauthorized("Missing X-API-Key header")),
            };
            
            let key_str = match key_val.to_str() {
                Ok(s) => s,
                Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid API Key format")),
            };

            let mut hasher = Sha256::new();
            hasher.update(key_str.as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            let data = match req.app_data::<web::Data<AppState>>() {
                Some(d) => d,
                None => return Err(actix_web::error::ErrorInternalServerError("State not found")),
            };

            // Using Id::new() is just for type hinting in query_as! macros sometimes needed but here query! infers types
            // However, return type of query! needs to match Id.
            // Since Id implements Decode for BIGINT (i64), let's see.
            // If the table column is BIGINT, sqlx returns i64.
            // Check if I can map it directly.
            // Usually sqlx::query! returns anonymous struct with primitive types.
            // So `record.id` will be i64.
            // I need to convert it to Id.
            
            let record = match sqlx::query!(
                "SELECT id, developer_id FROM api_keys WHERE key_hash = $1",
                hash
            )
            .fetch_optional(&data.db)
            .await {
                Ok(Some(r)) => r,
                Ok(None) => return Err(actix_web::error::ErrorUnauthorized("Invalid API Key")),
                Err(e) => {
                    log::error!("API Key DB Error: {}", e);
                    return Err(actix_web::error::ErrorInternalServerError("Database error"));
                }
            };
            
            Ok(ApiKeyMiddleware {
                developer_id: Id::from(record.developer_id),
                api_key_id: Id::from(record.id),
            })
        })
    }
}
