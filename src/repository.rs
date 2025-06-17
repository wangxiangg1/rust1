use diesel::prelude::*;
use crate::db::establish_connection;
use crate::models::{User, NewUser, Credential, NewCredential, ApiToken, NewApiToken};
use crate::schema::{users, credentials, api_tokens};
use anyhow::Result;
use uuid::Uuid;

// --- User Management ---
pub fn get_user_by_username(uname: &str) -> Result<User> {
    let conn = &mut establish_connection();
    users::table.filter(users::username.eq(uname)).first::<User>(conn).map_err(Into::into)
}

pub fn create_user(uname: &str, pwhash: &str) -> Result<User> {
    let conn = &mut establish_connection();
    let new_user = NewUser { username: uname, password_hash: pwhash };
    diesel::insert_into(users::table).values(&new_user).get_result(conn).map_err(Into::into)
}

pub fn ensure_admin_exists() -> Result<Option<String>> {
    if get_user_by_username("admin").is_ok() {
        return Ok(None);
    }
    let initial_password = crate::utils::generate_random_password(12);
    let hashed_password = crate::utils::hash_password(&initial_password)?;
    create_user("admin", &hashed_password)?;
    Ok(Some(initial_password))
}

// --- Credential Management ---
pub fn list_credentials() -> Result<Vec<Credential>> {
    let conn = &mut establish_connection();
    credentials::table.load::<Credential>(conn).map_err(Into::into)
}

pub fn create_credential(email_val: &str, token_val: &str) -> Result<Credential> {
    let conn = &mut establish_connection();
    let new_credential = NewCredential { email: email_val, token: token_val };
    diesel::insert_into(credentials::table).values(&new_credential).get_result(conn).map_err(Into::into)
}

pub fn delete_credential(cred_id: i32) -> Result<()> {
    let conn = &mut establish_connection();
    diesel::delete(credentials::table.find(cred_id)).execute(conn)?;
    Ok(())
}

// --- API Token Management ---
pub fn get_api_token(token_val: &str) -> Result<ApiToken> {
    let conn = &mut establish_connection();
    api_tokens::table.filter(api_tokens::token.eq(token_val)).first::<ApiToken>(conn).map_err(Into::into)
}

pub fn create_api_token() -> Result<ApiToken> {
    let conn = &mut establish_connection();
    let new_token_value = Uuid::new_v4().to_string();
    let new_api_token = NewApiToken { token: new_token_value };
    diesel::insert_into(api_tokens::table).values(&new_api_token).get_result(conn).map_err(Into::into)
}
