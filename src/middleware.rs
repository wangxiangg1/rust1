use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::auth::validate_token;
use std::future::Future;
use std::pin::Pin;

/// Bearer 认证验证器
pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    match validate_token(credentials.token()) {
        Ok(_claims) => Ok(req),
        Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
    }
}

// Helper to create middleware instance
pub fn jwt() -> HttpAuthentication<
    BearerAuth,
    impl Fn(
        ServiceRequest,
        BearerAuth,
    ) -> Pin<Box<dyn Future<Output = Result<ServiceRequest, Error>>>>,
> {
    HttpAuthentication::bearer(|req, creds| Box::pin(validator(req, creds)))
}
