use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};
mod db;
mod auth;
mod models;
mod repository;
mod middleware;
mod utils;
mod schema;
mod handlers;
mod admin_handlers;
mod services;

use auth::auth_handler;
use serde_json::json;
use repository::ensure_admin_exists;
use middleware::jwt;
use handlers::{list_models, chat_completions, health};
use reqwest::Client;
use admin_handlers::{show_login, handle_login, show_credentials, add_credential, delete_credential, generate_api_token};
use tera::Tera;
use actix_files as fs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let _conn = db::establish_connection();
    // 确保存在 admin 用户
    if let Ok(Some(initial_pwd)) = ensure_admin_exists() {
        println!("🔐 初始管理员密码: {} (请及时修改)", initial_pwd);
    }
    
    let tera = Tera::new("templates/**/*").expect("Error parsing templates");
    let client = Client::new();

    HttpServer::new(move || {
        let tera = tera.clone();
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(client.clone()))
            .wrap(Logger::default())
            .app_data(web::JsonConfig::default().error_handler(|err, _| {
                actix_web::error::InternalError::from_response(
                    err,
                    HttpResponse::BadRequest().json(json!({ "error": "Invalid request" }))
                ).into()
            }))
            .route("/", web::get().to(|| async { "Atlassian Rust Docker" }))
            .service(fs::Files::new("/static", "static").show_files_listing())
            .service(
                web::scope("/admin")
                    .route("/login", web::get().to(show_login))
                    .route("/login", web::post().to(handle_login))
                    .route("/credentials", web::get().to(show_credentials))
                    .route("/credentials", web::post().to(add_credential))
                    .route("/credential/{id}/delete", web::post().to(delete_credential))
                    .route("/api_token/generate", web::post().to(generate_api_token))
            )
            .service(
                web::scope("/api")
                    .route("/auth", web::post().to(auth_handler))
                    .route("/health", web::get().to(health))
                    .service(
                        web::scope("")
                            .wrap(jwt())
                            .route("/v1/models", web::get().to(list_models))
                            .route("/v1/chat/completions", web::post().to(chat_completions))
                            .route("/protected", web::get().to(|| async { "Protected route" }))
                    )
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
