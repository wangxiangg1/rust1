use crate::auth::validate_token;
use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use std::future::Future;

/// Bearer 认证验证器
pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    if validate_token(credentials.token()).is_ok() {
        Ok(req)
    } else {
        Err((actix_web::error::ErrorUnauthorized("Invalid token"), req))
    }
}

// Helper to create middleware instance
type ValidatorFuture = std::pin::Pin<Box<dyn Future<Output = Result<ServiceRequest, (Error, ServiceRequest)>> + 'static>>;

pub fn jwt() -> HttpAuthentication<BearerAuth, fn(ServiceRequest, BearerAuth) -> ValidatorFuture> {
    HttpAuthentication::bearer(|req, creds| Box::pin(validator(req, creds)))
}
