use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel::Connection;
use diesel::connection::SimpleConnection;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    let mut conn = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // 在首次连接时执行一次性迁移
    run_migrations(&mut conn);

    conn
}

/// 运行简化版迁移：若表不存在则创建
pub fn run_migrations(conn: &mut PgConnection) {
    let sql = r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR NOT NULL UNIQUE,
            password_hash VARCHAR NOT NULL
        );
        CREATE TABLE IF NOT EXISTS credentials (
            id SERIAL PRIMARY KEY,
            email VARCHAR NOT NULL,
            token TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS api_tokens (
            id SERIAL PRIMARY KEY,
            token TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
        );
    "#;

    conn.batch_execute(sql).expect("Failed to run migrations");
}
