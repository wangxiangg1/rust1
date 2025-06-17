//! 高层服务函数，封装数据库访问逻辑，供 handler 调用

use anyhow::Result;
use diesel::prelude::*;
use uuid::Uuid;

use crate::{db::establish_connection, models::{Credential, NewCredential, ApiToken, NewApiToken}};

// ----------------- Credential -----------------

pub fn list_credentials() -> Result<Vec<Credential>> {
    use crate::schema::credentials::dsl::*;
    let conn = &mut establish_connection();
    Ok(credentials.load::<Credential>(conn)?)
}

pub fn create_credential(email_str: &str, token_str: &str) -> Result<()> {
    use crate::schema::credentials::dsl::*;
    let conn = &mut establish_connection();
    let new = NewCredential { email: email_str.into(), token: token_str.into() };
    diesel::insert_into(credentials).values(&new).execute(conn)?;
    Ok(())
}

pub fn remove_credential(cid: i32) -> Result<()> {
    use crate::schema::credentials::dsl::*;
    let conn = &mut establish_connection();
    diesel::delete(credentials.filter(id.eq(cid))).execute(conn)?;
    Ok(())
}

// ----------------- API Token -----------------
pub fn current_api_token() -> Result<Option<ApiToken>> {
    use crate::schema::api_tokens::dsl::*;
    let conn = &mut establish_connection();
    Ok(api_tokens.order(id.desc()).first::<ApiToken>(conn).optional()?)
}

pub fn generate_api_token() -> Result<String> {
    use crate::schema::api_tokens::dsl::*;
    let conn = &mut establish_connection();
    let new_value = Uuid::new_v4().to_string();
    let new_row = NewApiToken { token: new_value.clone() };
    diesel::insert_into(api_tokens).values(&new_row).execute(conn)?;
    Ok(new_value)
}

pub fn revoke_api_token(token_id: i32) -> Result<()> {
    use crate::schema::api_tokens::dsl::*;
    let conn = &mut establish_connection();
    diesel::delete(api_tokens.filter(id.eq(token_id))).execute(conn)?;
    Ok(())
}

pub fn validate_api_token(token_str: &str) -> Result<()> {
    use crate::schema::api_tokens::dsl::*;
    let conn = &mut establish_connection();
    let token_found = diesel::select(diesel::dsl::exists(api_tokens.filter(token.eq(token_str))))
        .get_result::<bool>(conn)?;

    if token_found {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Invalid API token"))
    }
}
