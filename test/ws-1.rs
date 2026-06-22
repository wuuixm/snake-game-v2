
// snake-game-server/src/ws.rs
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use futures_util::{StreamExt, SinkExt};
use common::{ClientMessage, ServerMessage, RoomPlayer};
use crate::AppState;
use crate::room::{GameRoom, RoomState};

// WebSocket 升级路由处理器
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

// 处理具体每一个长连接
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // 局部变量：用来记录当前连接对应的玩家是谁，在哪个房间
    let mut current_username: Option<String> = None;
    let mut current_room_code: Option<String> = None;

    // 1. 网络输入循环：高频监听这个客户端发来的网络数据包
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            // 解析 JSON 报文到我们 common 里定义好的 ClientMessage 枚举
            let client_msg: ClientMessage = match serde_json::from_str(&text) {
                Ok(m) => m,
                Err(_) => {
                    let _ = sender.send(Message::Text(serde_json::to_string(&ServerMessage::Error {
                        message: "非法报文格式".to_string()
                    }).unwrap())).await;
                    continue;
                }
            };

            match client_msg {
                // --- 业务逻辑 1：加入房间（若不存在则创建） ---
                ClientMessage::JoinRoom { username } => {
                    // 局域网简单起见，暂定固定房间码 "8888" 或让客户端传入，这里我们硬编码一个演示码
                    let room_code = "8888".to_string(); 
                    current_username = Some(username.clone());
                    current_room_code = Some(room_code.clone());

                    let mut rooms = state.rooms.write().await;
                    // 如果房间不存在，动态创建它
                    let room_arc = rooms.entry(room_code.clone()).or_insert_with(|| {
                        Arc::new(RwLock::new(GameRoom::new(room_code.clone())))
                    }).clone();

                    let mut room = room_arc.write().await;
                    room.add_player(username.clone());

                    // 订阅这个房间的广播通道！这样后面服务器广播游戏帧时，这个连接才能收得到
                    let mut rx = room.tx.subscribe();
                    
                    // 广播通知房间内所有人：有人进来了
                    let room_status = ServerMessage::RoomStatus {
                        room_code: room_code.clone(),
                        players: room.players.values().cloned().collect(),
                        tick_rate_ms: room.tick_rate_ms,
                        is_all_ready: false,
                    };
                    let _ = room.tx.send(room_status);

                    // 另起一个异步任务：把这个房间广播通道里的数据，无脑转发给当前客户端
                    let mut sender_clone = room.tx.subscribe();
                    tokio::spawn(async move {
                        // 只要长连接还在，房间一广播，我们就通过 WebSockect 发送给客户端
                        while let Ok(server_msg) = sender_clone.recv().await {
                            let json_str = serde_json::to_string(&server_msg).unwrap();
                            if sender.send(Message::Text(json_str)).await.is_err() {
                                break; // 客户端断开连接，退出转发任务
                            }
                        }
                    });
                }

                // --- 业务逻辑 2：设置速度 ---
                ClientMessage::SetSpeed { tick_rate_ms } => {
                    if let Some(ref r_code) = current_room_code {
                        if let Some(room_arc) = state.rooms.read().await.get(r_code) {
                            let mut room = room_arc.write().await;
                            room.tick_rate_ms = tick_rate_ms;
                        }
                    }
                }

                // --- 业务逻辑 3：准备开局 ---
                ClientMessage::Ready => {
                    if let (Some(ref u_name), Some(ref r_code)) = (&current_username, &current_room_code) {
                        let rooms_guard = state.rooms.read().await;
                        if let Some(room_arc) = rooms_guard.get(r_code) {
                            let mut room = room_arc.write().await;
                            let is_all_ready = room.ready_player(u_name);

                            // 通知所有人准备状态更新
                            let _ = room.tx.send(ServerMessage::RoomStatus {
                                room_code: room.room_code.clone(),
                                players: room.players.values().cloned().collect(),
                                tick_rate_ms: room.tick_rate_ms,
                                is_all_ready,
                            });

                            // 如果全员准备好了，且游戏还没开始，正式激活独立的游戏主循环时钟引擎！
                            if is_all_ready {
                                room.init_game();
                                let _ = room.tx.send(ServerMessage::GameStart);
                                
                                // 克隆房间指针，抛给 Tokio 线程去跑死循环计算
                                let room_arc_for_loop = room_arc.clone();
                                let state_pool_clone = state.db_pool.clone();
                                
                                tokio::spawn(async move {
                                    let mut interval_ms = {
                                        let r = room_arc_for_loop.read().await;
                                        r.tick_rate_ms
                                    };
                                    let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
                                    
                                    loop {
                                        interval.tick().await; // 等待下一帧时钟
                                        
                                        let mut room = room_arc_for_loop.write().await;
                                        // 调用 room.rs 里的核心“计算函数”
                                        let keep_going = room.tick();
                                        
                                        if let Some(ref gs) = room.game_state {
                                            // 广播当前帧的画面数据给所有的前端和手柄
                                            let _ = room.tx.send(ServerMessage::GameFrame(gs.clone()));
                                        }

                                        // 游戏结束了，跳出时钟死循环，并写入 PostgreSQL 数据库
                                        if !keep_going {
                                            room.state = RoomState::Waiting;
                                            
                                            // 收集本局最终分数
                                            let mut round_scores = Vec::new();
                                            if let Some(ref gs) = room.game_state {
                                                for snake in &gs.snakes {
                                                    let score = (snake.body.len() as u32).saturating_sub(3) * 10; // 吃一个食物 10 分
                                                    round_scores.push((snake.username.clone(), score));
                                                    
                                                    // 异步写入数据库的 leaderboard 表
                                                    let _ = sqlx::query!(
                                                        "INSERT INTO leaderboard (username, score) VALUES ($1, $2)",
                                                        snake.username,
                                                        score as i32
                                                    ).execute(&state_pool_clone).await;
                                                }
                                            }

                                            // 广播游戏结束及结算数据
                                            let _ = room.tx.send(ServerMessage::GameOver {
                                                winner_username: room.players.keys().next().cloned(), // 简单示例：取第一个
                                                round_scores,
                                            });
                                            break; 
                                        }
                                    }
                                });
                            }
                        }
                    }
                }

                // --- 业务逻辑 4：实时改变方向 ---
                ClientMessage::ChangeDirection { direction } => {
                    if let (Some(ref u_name), Some(ref r_code)) = (&current_username, &current_room_code) {
                        if let Some(room_arc) = state.rooms.read().await.get(r_code) {
                            let mut room = room_arc.write().await;
                            if let Some(ref mut gs) = room.game_state {
                                if let Some(snake) = gs.snakes.iter_mut().find(|s| s.username == *u_name) {
                                    // 防止回头撞死逻辑（比如当前向右，按左无效）
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

                // --- 业务逻辑 5：玩家中途主动退出 ---
                ClientMessage::LeaveRoom => {
                    if let (Some(ref u_name), Some(ref r_code)) = (&current_username, &current_room_code) {
                        let mut rooms = state.rooms.write().await;
                        if let Some(room_arc) = rooms.get(r_code) {
                            let mut room = room_arc.write().await;
                            room.remove_player(u_name);
                            // 如果人都走光了，顺手把房间从外层大 HashMap 删掉释放内存
                            if room.players.is_empty() {
                                rooms.remove(r_code);
                            }
                        }
                    }
                    break; // 跳出接收死循环，断开长连接
                }
                ClientMessage::Ping => {}
                _ => {}
            }
        }
    }

    // --- 网络保底清理：如果客户端直接拔掉网线或者强关程序 ---
    if let (Some(ref u_name), Some(ref r_code)) = (current_username, current_room_code) {
        let mut rooms = state.rooms.write().await;
        if let Some(room_arc) = rooms.get(r_code) {
            let mut room = room_arc.write().await;
            room.remove_player(u_name);
            if room.players.is_empty() {
                rooms.remove(r_code);
            }
        }
    }
}