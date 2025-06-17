use actix_web::{web, HttpResponse, HttpRequest, Responder};
use tera::{Context, Tera};
use crate::{repository, services, utils, auth};
use serde::Deserialize;

/// 显示管理员登录页面
pub async fn show_login(tmpl: web::Data<Tera>) -> impl Responder {
    let ctx = Context::new();
    let rendered = tmpl
        .render("login.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered)
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

/// 处理登录表单
pub async fn handle_login(form: web::Form<LoginForm>, tmpl: web::Data<Tera>) -> impl Responder {
    match repository::get_user_by_username(&form.username) {
        Ok(user) => {
            if utils::verify_password(&user.password_hash, &form.password).unwrap_or(false) {
                // 生成 JWT 并写 Cookie
                let token = auth::generate_token(&user.username).unwrap();
                HttpResponse::Found()
                    .append_header(("Location", "/admin/credentials"))
                    .cookie(
                        actix_web::cookie::Cookie::build("admin_jwt", token)
                            .path("/")
                            .http_only(true)
                            .finish(),
                    )
                    .finish()
            } else {
                render_error(&tmpl, "密码错误")
            }
        }
        Err(_) => render_error(&tmpl, "用户不存在"),
    }
}

/// 显示凭据列表
pub async fn show_credentials(req: HttpRequest, tmpl: web::Data<Tera>) -> impl Responder {
    if !check_cookie(&req) {
        return HttpResponse::Found().append_header(("Location", "/admin/login")).finish();
    }
    let creds = services::list_credentials().unwrap_or_default();
    let api = services::current_api_token().unwrap_or(None);

    let mut ctx = Context::new();
    ctx.insert("credentials", &creds);
    ctx.insert("api_token", &api);

    let rendered = tmpl
        .render("credentials.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered)
}

#[derive(Deserialize)]
pub struct CredentialForm {
    email: String,
    token: String,
}

pub async fn add_credential(req: HttpRequest, form: web::Form<CredentialForm>) -> impl Responder {
    if !check_cookie(&req) { return HttpResponse::Found().append_header(("Location", "/admin/login")).finish(); }

    if let Err(e) = services::create_credential(&form.email, &form.token) {
        return HttpResponse::InternalServerError().body(format!("Error: {e}"));
    }
    HttpResponse::Found()
        .append_header(("Location", "/admin/credentials"))
        .finish()
}

pub async fn delete_credential(req: HttpRequest, path: web::Path<i32>) -> impl Responder {
    if !check_cookie(&req) { return HttpResponse::Found().append_header(("Location", "/admin/login")).finish(); }

    let cid = path.into_inner();
    let _ = services::remove_credential(cid);
    HttpResponse::Found()
        .append_header(("Location", "/admin/credentials"))
        .finish()
}

pub async fn generate_api_token(req: HttpRequest) -> impl Responder {
    if !check_cookie(&req) { return HttpResponse::Found().append_header(("Location", "/admin/login")).finish(); }

    let _ = services::generate_api_token();
    HttpResponse::Found()
        .append_header(("Location", "/admin/credentials"))
        .finish()
}

fn check_cookie(req: &HttpRequest) -> bool {
    if let Some(cookie) = req.cookie("admin_jwt") {
        return auth::validate_token(cookie.value()).is_ok();
    }
    false
}

fn render_error(tmpl: &Tera, msg: &str) -> HttpResponse {
    let mut ctx = Context::new();
    ctx.insert("error", msg);
    let rendered = tmpl
        .render("error.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered)
}
