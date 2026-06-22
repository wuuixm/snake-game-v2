// snake-game-server/src/main.rs
use axum::{routing::{any, get, post}, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

mod room;
mod ws;
mod handlers;
mod game;

use ws::ws_handler;

// 定义全局共享状态
pub struct AppState {
    pub rooms: tokio::sync::RwLock<std::collections::HashMap<String, Arc<tokio::sync::RwLock<room::GameRoom>>>>,
    pub db_pool: sqlx::PgPool, // PostgreSQL 连接池
}

#[tokio::main]
async fn main() {
    // 1. 初始化数据库连接池
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://pgsql:123456@localhost:5432/snake_db".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("无法连接到 PostgreSQL 数据库，请检查数据库是否启动且库名正确");

    // 自动建表——列名与 db.sql 严格一致
    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL
         );"
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "CREATE TABLE IF NOT EXISTS leaderboard (
            id SERIAL PRIMARY KEY,
            username TEXT NOT NULL,
            score INT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT NOW()
         );"
    )
    .execute(&pool)
    .await
    .unwrap();

    let state = Arc::new(AppState {
        rooms: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        db_pool: pool,
    });

    // 2. 配置跨域规则
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 3. 构建路由中枢 — HTTP 鉴权指向 handlers.rs
    let app = Router::new()
        .route("/ws", any(ws_handler))
        .route("/api/leaderboard", get(handlers::leaderboard_handler))
        .route("/api/auth/register", post(handlers::register_handler))
        .route("/api/auth/login", post(handlers::login_handler))
        .layer(cors)
        .with_state(state);

    // 绑定 0.0.0.0 以允许局域网内其他设备通过 WiFi 访问
    let bind_host = std::env::var("BIND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let bind_port = std::env::var("BIND_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("{}:{}", bind_host, bind_port)
        .parse()
        .expect("无效的绑定地址");
    println!("🚀 贪吃蛇服务端已启动 → http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
