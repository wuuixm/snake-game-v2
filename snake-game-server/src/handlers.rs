use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;
// 客户端提交注册/登录时的 HTTP 请求体
#[derive(Deserialize)]
pub struct AuthRequest {
    username: String,
    password_hash: String, // 前端传过来的原始密码（或者是前端初步哈希过的）
}

// 统一的 HTTP 响应体
#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}
// --- 账户注册 ---
pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthRequest>,
) -> impl IntoResponse {
    // 1. 检查用户名长度等基础合法性
    if payload.username.trim().is_empty() || payload.password_hash.len() < 6 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()> {
                success: false,
                message: "用户名不能为空且密码不能少于6位".to_string(),
                data: None,
            }),
        );
    }

    // 2. 使用 bcrypt 在服务端对密码进行强加密
    let hashed_password = match bcrypt::hash(&payload.password_hash, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse { success: false, message: "密码加密失败".to_string(), data: None })
        ),
    };

    // 3. 写入 PostgreSQL 数据库
    // let result = sqlx::query!(
    //     "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
    //     payload.username,
    //     hashed_password
    // )
    // .execute(&state.db_pool)
    // .await;
    let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
        payload.username,
        hashed_password
    )
    .execute(&state.db_pool)
    .await; 

    match result {
        Ok(_) => (
            StatusCode::CREATED,
            Json(ApiResponse::<()> {
                success: true,
                message: "注册成功".to_string(),
                data: None,
            }),
        ),
        Err(e) => {
            // 如果报错通常是用户名重复（Unique 约束触发）
            let msg = if e.to_string().contains("unique constraint") {
                "该用户名已被注册"
            } else {
                "数据库写入失败"
            };
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse { success: false, message: msg.to_string(), data: None }),
            )
        }
    }
}

// --- 账户登录 ---
pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthRequest>,
) -> impl IntoResponse {
    // 1. 根据用户名去数据库查密文
    let user_record = sqlx::query!(
        "SELECT password_hash FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&state.db_pool)
    .await;

    match user_record {
        Ok(Some(row)) => {
            // 2. 用 bcrypt 验证密码是否匹配
            match bcrypt::verify(&payload.password_hash, &row.password_hash) {
                Ok(true) => (
                    StatusCode::OK,
                    Json(ApiResponse::<()> {
                        success: true,
                        message: "登录成功".to_string(),
                        data: None,
                    }),
                ),
                _ => (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse { success: false, message: "密码错误".to_string(), data: None }),
                ),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse { success: false, message: "用户不存在".to_string(), data: None }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse { success: false, message: "数据库查询失败".to_string(), data: None }),
        ),
    }
}

// --- 全局积分榜查询 ---
pub async fn leaderboard_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 1. 执行我们在第三步讨论的聚合查询 SQL
    // 注：因为我们要映射成 common 里的 LeaderboardEntry，可以用 sqlx::query_as!
    let rows: Result<Vec<_>, sqlx::Error> = sqlx::query!(
        r#"
        SELECT 
            username,
            MAX(score) as "max_score!",
            COUNT(*) as "total_games!"
        FROM leaderboard
        GROUP BY username
        ORDER BY MAX(score) DESC  -- ← 这里改用原始聚合函数，避免别名解析冲突
        LIMIT 10
        "#
    )
    .fetch_all(&state.db_pool)
    .await;

    match rows {
        Ok(records) => {
            // 2. 将数据组装成 common::LeaderboardEntry 列表
            let mut leaderboard_entries = Vec::new();
            for (index, row) in records.into_iter().enumerate() {
                leaderboard_entries.push(common::LeaderboardEntry {
                    rank: index + 1,
                    username: row.username,
                    max_score: row.max_score as u32,
                    total_games: row.total_games as u32,
                });
            }
            (
                StatusCode::OK,
                Json(ApiResponse {
                    success: true,
                    message: "获取积分榜成功".to_string(),
                    data: Some(leaderboard_entries),
                }),
            )
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse { success: false, message: "获取积分榜失败".to_string(), data: None }),
        ),
    }
}
