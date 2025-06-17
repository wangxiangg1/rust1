use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use crate::schema::{users, credentials, api_tokens};

#[derive(Queryable, Identifiable, Serialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
}

#[derive(Queryable, Identifiable, Serialize)]
#[diesel(table_name = credentials)]
pub struct Credential {
    pub id: i32,
    pub email: String,
    pub token: String,
}

#[derive(Insertable)]
#[diesel(table_name = credentials)]
pub struct NewCredential<'a> {
    pub email: &'a str,
    pub token: &'a str,
}

#[derive(Queryable, Identifiable, Serialize)]
#[diesel(table_name = api_tokens)]
pub struct ApiToken {
    pub id: i32,
    pub token: String,
}

#[derive(Insertable)]
#[diesel(table_name = api_tokens)]
pub struct NewApiToken {
    pub token: String,
}
