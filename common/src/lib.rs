use serde::{Deserialize, Serialize};

// ==========================================
// 0. 游戏模式（经典 / 锦标赛）
// ==========================================
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    Classic,
    Tournament,
}

impl Default for GameMode {
    fn default() -> Self {
        GameMode::Classic
    }
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMode::Classic => write!(f, "Classic"),
            GameMode::Tournament => write!(f, "Tournament"),
        }
    }
}

// ==========================================
// 0b. 食物类型（不同外观 + 不同分值）
// ==========================================
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum FoodType {
    Apple,        // 10 分 — 红色苹果
    Cherry,       // 20 分 — 樱桃
    Orange,       // 15 分 — 橙子
    Grape,        // 25 分 — 葡萄
    Melon,        // 50 分 — 蜜瓜（稀有）
    SnakeRemains, // 15 分 — 蛇死亡后遗留的食物
}

impl FoodType {
    /// 返回该食物类型对应的分值
    pub fn score_value(&self) -> u32 {
        match self {
            FoodType::Apple => 10,
            FoodType::Cherry => 20,
            FoodType::Orange => 15,
            FoodType::Grape => 25,
            FoodType::Melon => 50,
            FoodType::SnakeRemains => 15,
        }
    }

    /// 返回该食物类型的字符串标识（供客户端 SVG 映射）
    pub fn as_str(&self) -> &str {
        match self {
            FoodType::Apple => "Apple",
            FoodType::Cherry => "Cherry",
            FoodType::Orange => "Orange",
            FoodType::Grape => "Grape",
            FoodType::Melon => "Melon",
            FoodType::SnakeRemains => "SnakeRemains",
        }
    }
}

// ==========================================
// 1. 客户端发送给服务端的统一报文 (ClientMessage)
// ==========================================
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    // --- 账户相关（走 HTTP，不走 WS；保留兼容） ---
    Register {
        username: String,
        password_hash: String,
    },
    Login {
        username: String,
        password_hash: String,
    },

    // --- 房间管理 ---
    /// 创建房间并自动加入为房主（room_code 可选，不传则自动生成 4 位码）
    CreateRoom {
        #[serde(skip_serializing_if = "Option::is_none")]
        room_code: Option<String>,
        username: String,
        color: String,
        #[serde(default = "default_game_mode")]
        mode: String,
    },
    /// 请求可加入的房间列表
    ListRooms,
    /// 加入指定房间
    JoinRoom {
        room_code: String,
        username: String,
        color: String,
        #[serde(default = "default_game_mode")]
        mode: String,
    },
    /// 请求刷新当前房间状态
    RefreshRoom,

    // --- 房间与游戏控制 ---
    SetSpeed {
        tick_rate_ms: u64,
    }, // 房主调整游戏速度（单位：毫秒/帧）
    Ready,     // 玩家准备
    LeaveRoom, // 玩家主动退出房间

    // --- 游戏内实时操作 ---
    ChangeDirection {
        direction: Direction,
    }, // 改变方向
    Forfeit, // 游戏内投降（蛇死亡，留在房间观战）

    // --- 心跳 ---
    Ping,
}

fn default_game_mode() -> String {
    "Classic".to_string()
}

// ==========================================
// 2. 服务端广播/回复给客户端的统一报文 (ServerMessage)
// ==========================================
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    // --- 账户操作反馈 ---
    AuthResult {
        success: bool,
        message: String,
    },

    // --- 房间状态同步 ---
    RoomStatus {
        room_code: String,
        players: Vec<RoomPlayer>,
        tick_rate_ms: u64,
        is_all_ready: bool,
        mode: String,
        host_username: String,
    },

    // --- 房间列表 ---
    RoomList {
        rooms: Vec<RoomInfo>,
    },

    // --- 游戏流程 ---
    GameStart,
    GameFrame(GameState),

    // --- 游戏结算 ---
    GameOver {
        winner_username: Option<String>,
        round_scores: Vec<(String, u32)>,
    },

    // --- 全局积分榜 ---
    Leaderboard(Vec<LeaderboardEntry>),

    // --- 锦标赛 ---
    TournamentStage(TournamentStageInfo),
    TournamentResult(TournamentResultInfo),

    // --- 异常与心跳 ---
    Error {
        message: String,
    },
    Pong,
}

// ==========================================
// 3. 辅助子结构体 (Helper Structs)
// ==========================================

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomPlayer {
    pub username: String,
    pub is_ready: bool,
    pub color: String,
}

/// 房间列表项（供大厅展示）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomInfo {
    pub room_code: String,
    pub player_count: usize,
    pub max_players: usize,
    pub mode: String,
    pub host_username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FoodItem {
    pub position: Position,
    pub food_type: FoodType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub snakes: Vec<Snake>,
    pub foods: Vec<FoodItem>,
    pub tick_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snake {
    pub username: String,
    pub body: Vec<Position>,
    pub current_dir: Direction,
    pub is_alive: bool,
    pub color: String,
    /// 本局累计得分（吃食物累积）
    #[serde(default)]
    pub score: u32,
    /// 刚死亡标记（用于生成 SnakeRemains 食物，仅服务端使用）
    #[serde(default)]
    pub just_died: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub username: String,
    pub max_score: u32,
    pub total_games: u32,
}

// ==========================================
// 4. 锦标赛结构体
// ==========================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TournamentStageInfo {
    pub stage: String,
    pub active_players: Vec<String>,
    pub bracket_a: (String, String),
    pub bracket_b: (String, String),
    pub winner_a: Option<String>,
    pub winner_b: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TournamentResultInfo {
    pub champion: String,
    pub runner_up: String,
    pub rankings: Vec<(String, usize)>,
}
