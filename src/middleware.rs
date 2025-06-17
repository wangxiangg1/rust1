use crate::auth::validate_token;
use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};

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
pub fn jwt() -> HttpAuthentication<BearerAuth> {
    HttpAuthentication::bearer(validator)
}
