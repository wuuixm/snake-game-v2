// snake-game-server/src/ws.rs
use crate::room::{GameRoom, RoomState};
use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use common::{ClientMessage, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;

// WebSocket 升级路由处理器
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// 构建 RoomStatus 消息的辅助函数
fn build_room_status(room: &GameRoom) -> ServerMessage {
    ServerMessage::RoomStatus {
        room_code: room.room_code.clone(),
        players: room.players.values().cloned().collect(),
        tick_rate_ms: room.tick_rate_ms,
        is_all_ready: room.players.values().all(|p| p.is_ready),
        mode: room.mode.to_string(),
        host_username: room.host_username.clone(),
    }
}

// 处理具体每一个长连接
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    let mut current_username: Option<String> = None;
    let mut current_room_code: Option<String> = None;

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            let client_msg: ClientMessage = match serde_json::from_str(&text) {
                Ok(m) => m,
                Err(e) => {
                    let mut s = sender.lock().await;
                    let _ = s
                        .send(Message::Text(
                            serde_json::to_string(&ServerMessage::Error {
                                message: format!("非法报文格式: {}", e),
                            })
                            .unwrap(),
                        ))
                        .await;
                    continue;
                }
            };

            match client_msg {
                // ============================================================
                // CreateRoom — 创建房间（可选自定义房间码）
                // ============================================================
                ClientMessage::CreateRoom {
                    room_code,
                    username,
                    color,
                    mode,
                } => {
                    // 如果已在一个房间中，先退出
                    if let (Some(ref u_name), Some(ref r_code)) =
                        (&current_username, &current_room_code)
                    {
                        let mut rooms = state.rooms.write().await;
                        if let Some(room_arc) = rooms.get(r_code).cloned() {
                            let mut room = room_arc.write().await;
                            room.remove_player(u_name);
                            if room.players.is_empty() {
                                rooms.remove(r_code);
                            }
                        }
                    }

                    let new_code = room_code
                        .filter(|c| !c.trim().is_empty() && c.len() <= 8)
                        .unwrap_or_else(GameRoom::generate_room_code);

                    // 确保房间码不重复
                    let mut rooms = state.rooms.write().await;
                    let final_code = if rooms.contains_key(&new_code) {
                        GameRoom::generate_room_code()
                    } else {
                        new_code
                    };

                    let room_arc =
                        Arc::new(tokio::sync::RwLock::new(GameRoom::new(final_code.clone())));
                    rooms.insert(final_code.clone(), room_arc.clone());

                    let game_mode = match mode.as_str() {
                        "Tournament" => common::GameMode::Tournament,
                        _ => common::GameMode::Classic,
                    };

                    // 自动将创建者加入为房主
                    {
                        let mut room = room_arc.write().await;
                        room.add_player(username.clone(), color, game_mode);
                    }

                    current_username = Some(username);
                    current_room_code = Some(final_code.clone());

                    // 订阅广播
                    let mut broadcast_rx = {
                        let room = room_arc.read().await;
                        room.tx.subscribe()
                    };
                    let sender_clone = sender.clone();

                    tokio::spawn(async move {
                        while let Ok(server_msg) = broadcast_rx.recv().await {
                            let json_str = serde_json::to_string(&server_msg).unwrap_or_default();
                            let mut s = sender_clone.lock().await;
                            if s.send(Message::Text(json_str)).await.is_err() {
                                break;
                            }
                        }
                    });

                    // 广播 RoomStatus
                    let status = {
                        let room = room_arc.read().await;
                        build_room_status(&room)
                    };
                    let _ = room_arc.read().await.tx.send(status);
                }

                // ============================================================
                // ListRooms — 列出所有可加入的房间
                // ============================================================
                ClientMessage::ListRooms => {
                    let rooms = state.rooms.read().await;
                    let room_list: Vec<_> = rooms
                        .values()
                        .filter_map(|r| {
                            let room = r.try_read().ok()?;
                            if room.state == RoomState::Waiting {
                                Some(room.to_room_info())
                            } else {
                                None
                            }
                        })
                        .collect();
                    drop(rooms);

                    let mut s = sender.lock().await;
                    let _ = s
                        .send(Message::Text(
                            serde_json::to_string(&ServerMessage::RoomList { rooms: room_list })
                                .unwrap(),
                        ))
                        .await;
                }

                // ============================================================
                // JoinRoom — 加入指定房间（携带玩家名、颜色、模式）
                // ============================================================
                ClientMessage::JoinRoom {
                    room_code,
                    username,
                    color,
                    mode,
                } => {
                    // 如果已在一个房间中，先退出
                    if let (Some(ref u_name), Some(ref r_code)) =
                        (&current_username, &current_room_code)
                    {
                        let mut rooms = state.rooms.write().await;
                        if let Some(room_arc) = rooms.get(r_code).cloned() {
                            let mut room = room_arc.write().await;
                            room.remove_player(u_name);
                            if room.players.is_empty() {
                                rooms.remove(r_code);
                            }
                        }
                    }

                    // 仅加入已有房间 — 不允许自动创建
                    let rooms = state.rooms.read().await;
                    let Some(room_arc) = rooms.get(&room_code).cloned() else {
                        let mut s = sender.lock().await;
                        let _ = s
                            .send(Message::Text(
                                serde_json::to_string(&ServerMessage::Error {
                                    message: format!("房间 {} 不存在，请先在客户端创建", room_code),
                                })
                                .unwrap(),
                            ))
                            .await;
                        continue;
                    };
                    drop(rooms);

                    let game_mode = match mode.as_str() {
                        "Tournament" => common::GameMode::Tournament,
                        _ => common::GameMode::Classic,
                    };

                    let joined = {
                        let mut room = room_arc.write().await;
                        room.add_player(username.clone(), color, game_mode)
                    };

                    if !joined {
                        let mut s = sender.lock().await;
                        let err_msg = {
                            let room = room_arc.read().await;
                            if room.state != RoomState::Waiting {
                                "房间游戏中，无法加入".to_string()
                            } else {
                                "房间已满（最多4人）".to_string()
                            }
                        };
                        let _ = s
                            .send(Message::Text(
                                serde_json::to_string(&ServerMessage::Error { message: err_msg })
                                    .unwrap(),
                            ))
                            .await;
                        continue;
                    }

                    current_username = Some(username);
                    current_room_code = Some(room_code.clone());

                    // 先创建广播订阅，再发送 RoomStatus
                    let mut broadcast_rx = {
                        let room = room_arc.read().await;
                        room.tx.subscribe()
                    };
                    let sender_clone = sender.clone();

                    tokio::spawn(async move {
                        while let Ok(server_msg) = broadcast_rx.recv().await {
                            let json_str = serde_json::to_string(&server_msg).unwrap_or_default();
                            let mut s = sender_clone.lock().await;
                            if s.send(Message::Text(json_str)).await.is_err() {
                                break;
                            }
                        }
                    });

                    // 广播 RoomStatus 给房间所有人（包括自己）
                    let status = {
                        let room = room_arc.read().await;
                        build_room_status(&room)
                    };
                    let _ = room_arc.read().await.tx.send(status);
                }

                // ============================================================
                // RefreshRoom — 请求当前房间状态（点对点，不广播）
                // ============================================================
                ClientMessage::RefreshRoom => {
                    if let Some(ref r_code) = current_room_code {
                        if let Some(room_arc) = state.rooms.read().await.get(r_code) {
                            let room = room_arc.read().await;
                            let status = build_room_status(&room);
                            let mut s = sender.lock().await;
                            let _ = s
                                .send(Message::Text(serde_json::to_string(&status).unwrap()))
                                .await;
                        }
                    }
                }

                // ============================================================
                // SetSpeed — 仅房主可调整游戏速度
                // ============================================================
                ClientMessage::SetSpeed { tick_rate_ms } => {
                    if let (Some(ref u_name), Some(ref r_code)) =
                        (&current_username, &current_room_code)
                    {
                        if let Some(room_arc) = state.rooms.read().await.get(r_code) {
                            let mut room = room_arc.write().await;
                            if room.is_host(u_name) {
                                room.tick_rate_ms = tick_rate_ms;
                                // 速度变更后广播 RoomStatus
                                let _ = room.tx.send(build_room_status(&room));
                            }
                        }
                    }
                }

                // ============================================================
                // Ready — 玩家准备（经典/锦标赛分流）
                // ============================================================
                ClientMessage::Ready => {
                    if let (Some(ref u_name), Some(ref r_code)) =
                        (&current_username, &current_room_code)
                    {
                        let rooms_guard = state.rooms.read().await;
                        if let Some(room_arc) = rooms_guard.get(r_code) {
                            let mut room = room_arc.write().await;
                            room.ready_player(u_name);

                            let _ = room.tx.send(build_room_status(&room));

                            if !room.can_start_game() {
                                continue;
                            }

                            match room.mode {
                                common::GameMode::Tournament => {
                                    let stage_info = room.start_tournament();
                                    let _ =
                                        room.tx.send(ServerMessage::TournamentStage(stage_info));
                                    let _ = room.tx.send(ServerMessage::GameStart);
                                    spawn_game_loop(room_arc.clone(), state.db_pool.clone(), true);
                                }
                                common::GameMode::Classic => {
                                    room.init_game();
                                    let _ = room.tx.send(ServerMessage::GameStart);
                                    spawn_game_loop(room_arc.clone(), state.db_pool.clone(), false);
                                }
                            }
                        }
                    }
                }

                // ============================================================
                // ChangeDirection — 实时方向控制（锦标赛中仅活跃玩家可操作）
                // ============================================================
                ClientMessage::ChangeDirection { direction } => {
                    if let (Some(ref u_name), Some(ref r_code)) =
                        (&current_username, &current_room_code)
                    {
                        if let Some(room_arc) = state.rooms.read().await.get(r_code) {
                            let room = room_arc.read().await;
                            if !room.can_player_move(u_name) {
                                continue;
                            }
                        }
                        if let Some(room_arc) = state.rooms.read().await.get(r_code) {
                            let mut room = room_arc.write().await;
                            if let Some(ref mut gs) = room.game_state {
                                if let Some(snake) =
                                    gs.snakes.iter_mut().find(|s| s.username == *u_name)
                                {
                                    // 防止 180 度掉头
                                    match (snake.current_dir, direction) {
                                        (common::Direction::Up, common::Direction::Down) => {}
                                        (common::Direction::Down, common::Direction::Up) => {}
                                        (common::Direction::Left, common::Direction::Right) => {}
                                        (common::Direction::Right, common::Direction::Left) => {}
                                        _ => snake.current_dir = direction,
                                    }
                                }
                            }
                        }
                    }
                }

                // ============================================================
                // Forfeit — 游戏内投降（蛇死亡，留在房间观战）
                // ============================================================
                ClientMessage::Forfeit => {
                    if let (Some(ref u_name), Some(ref r_code)) =
                        (&current_username, &current_room_code)
                    {
                        if let Some(room_arc) = state.rooms.read().await.get(r_code) {
                            let mut room = room_arc.write().await;
                            room.forfeit_player(u_name);
                        }
                    }
                }

                // ============================================================
                // LeaveRoom — 玩家主动退出（仅在 Room 状态可用）
                // ============================================================
                ClientMessage::LeaveRoom => {
                    if let (Some(ref u_name), Some(ref r_code)) =
                        (&current_username, &current_room_code)
                    {
                        let mut rooms = state.rooms.write().await;
                        if let Some(room_arc) = rooms.get(r_code).cloned() {
                            let new_host = {
                                let mut room = room_arc.write().await;
                                room.remove_player(u_name)
                            };
                            // 如果房主转移或房间还有人，广播 RoomStatus
                            if new_host.is_some() {
                                let room = room_arc.read().await;
                                if !room.players.is_empty() {
                                    let _ = room.tx.send(build_room_status(&room));
                                }
                            }
                            if room_arc.read().await.players.is_empty() {
                                rooms.remove(r_code);
                            }
                        }
                    }
                    break;
                }

                ClientMessage::Ping => {
                    let mut s = sender.lock().await;
                    let _ = s
                        .send(Message::Text(
                            serde_json::to_string(&ServerMessage::Pong).unwrap(),
                        ))
                        .await;
                }

                // 忽略未登录就发出的操作（Register/Login 走 HTTP）
                _ => {}
            }
        }
    }

    // ============================================================
    // 连接断开后的清理
    // ============================================================
    if let (Some(ref u_name), Some(ref r_code)) = (&current_username, &current_room_code) {
        let mut rooms = state.rooms.write().await;
        if let Some(room_arc) = rooms.get(r_code).cloned() {
            let is_playing = room_arc.read().await.state == RoomState::Playing;

            if is_playing {
                // 游戏中断线：标记蛇死亡，保留玩家在房间
                let mut room = room_arc.write().await;
                room.forfeit_player(u_name);
            } else {
                // 等待中断线：完整移除
                let new_host = {
                    let mut room = room_arc.write().await;
                    room.remove_player(u_name)
                };
                // 房主转移 → 广播
                if new_host.is_some() {
                    let room = room_arc.read().await;
                    if !room.players.is_empty() {
                        let _ = room.tx.send(build_room_status(&room));
                    }
                }
                if room_arc.read().await.players.is_empty() {
                    rooms.remove(r_code);
                }
            }
        }
    }
}

// ============================================================
// 游戏循环（经典混战 & 锦标赛共用）
// ============================================================
fn spawn_game_loop(
    room_arc: Arc<tokio::sync::RwLock<GameRoom>>,
    db_pool: sqlx::PgPool,
    is_tournament: bool,
) {
    tokio::spawn(async move {
        loop {
            let interval_ms = {
                let r = room_arc.read().await;
                r.tick_rate_ms
            };
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
            interval.tick().await; // 吞掉首个即时触发

            loop {
                interval.tick().await;

                let mut room = room_arc.write().await;
                let keep_going = room.tick();

                // 广播当前帧
                if let Some(ref gs) = room.game_state {
                    let _ = room.tx.send(ServerMessage::GameFrame(gs.clone()));
                }

                if keep_going {
                    continue;
                }

                // ---- 游戏结束 ----
                room.state = RoomState::Waiting;

                // 写入积分榜（使用蛇的累计 score）
                let mut round_scores = Vec::new();
                if let Some(ref gs) = room.game_state {
                    for snake in &gs.snakes {
                        let score = snake.score;
                        round_scores.push((snake.username.clone(), score));
                        if score > 0 {
                            let _ = sqlx::query!(
                                "INSERT INTO leaderboard (username, score) VALUES ($1, $2)",
                                snake.username,
                                score as i32
                            )
                            .execute(&db_pool)
                            .await;
                        }
                    }
                }

                let winner = room
                    .game_state
                    .as_ref()
                    .and_then(|gs| gs.snakes.iter().find(|s| s.is_alive))
                    .map(|s| s.username.clone());

                if is_tournament {
                    // 锦标赛：推进到下一阶段
                    match room.advance_tournament() {
                        Some(stage_info) => {
                            let _ = room.tx.send(ServerMessage::TournamentStage(stage_info));
                            let _ = room.tx.send(ServerMessage::GameStart);
                            continue; // 继续外循环，开始下一阶段游戏
                        }
                        None => {
                            // 锦标赛结束
                            if let Some(result) = room.tournament_result() {
                                let _ = room.tx.send(ServerMessage::TournamentResult(result));
                            }
                            let _ = room.tx.send(ServerMessage::GameOver {
                                winner_username: winner,
                                round_scores,
                            });
                            room.reset_ready();
                            room.tournament = None;
                            // Bug fix: 广播 RoomStatus 让客户端 ready 状态同步
                            let _ = room.tx.send(build_room_status(&room));
                            break;
                        }
                    }
                } else {
                    // 经典模式：直接结算
                    let _ = room.tx.send(ServerMessage::GameOver {
                        winner_username: winner,
                        round_scores,
                    });
                    room.reset_ready();
                    // Bug fix: 广播 RoomStatus 让客户端 ready 状态同步
                    let _ = room.tx.send(build_room_status(&room));
                    break;
                }
            }

            // 非锦标赛 → 外循环结束
            if !is_tournament {
                break;
            }
            // 锦标赛内循环的 continue 会回到这里重新读取 tick_rate_ms
        }
    });
}
