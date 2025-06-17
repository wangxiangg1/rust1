use actix_web::{HttpResponse, Responder, web};
use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use crate::{repository, utils};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    username: String,
    password: String,
}

pub fn validate_token(token: &str) -> Result<Claims> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

pub fn generate_token(user_id: &str) -> Result<String> {
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };
    
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "secret".into());
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    
    Ok(token)
}

pub async fn auth_handler(req: web::Json<AuthRequest>) -> impl Responder {
    match repository::get_user_by_username(&req.username) {
        Ok(user) => {
            if utils::verify_password(&user.password_hash, &req.password).unwrap_or(false) {
                match generate_token(&user.username) {
                    Ok(token) => HttpResponse::Ok().json(json!({ "token": token })),
                    Err(_) => HttpResponse::InternalServerError().finish(),
                }
            } else {
                HttpResponse::Unauthorized().finish()
            }
        }
        Err(_) => HttpResponse::Unauthorized().finish(),
    }
}
