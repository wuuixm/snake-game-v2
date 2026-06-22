// snake-game-server/src/room.rs
use std::collections::HashMap;
use tokio::sync::broadcast;
use common::{
    GameState, ServerMessage, RoomPlayer, RoomInfo,
    TournamentStageInfo, TournamentResultInfo,
};
use rand::Rng;

const MAX_PLAYERS: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub enum RoomState {
    Waiting,
    Playing,
}

// ============================================================
// 锦标赛阶段
// ============================================================
#[derive(Debug, Clone, PartialEq)]
pub enum TournamentStage {
    NotStarted,
    SemifinalA,
    SemifinalB,
    Final,
    Completed,
}

pub struct TournamentState {
    pub stage: TournamentStage,
    /// 按加入顺序排列的 4 个玩家名
    pub player_indices: Vec<String>,
    pub winner_a: Option<String>,
    pub winner_b: Option<String>,
    pub champion: Option<String>,
}

impl TournamentState {
    /// 当前阶段允许操控的玩家列表
    pub fn active_players(&self) -> Vec<String> {
        match self.stage {
            TournamentStage::NotStarted | TournamentStage::Completed => vec![],
            TournamentStage::SemifinalA => {
                let a = &self.player_indices;
                vec![a[0].clone(), a[1].clone()]
            }
            TournamentStage::SemifinalB => {
                let a = &self.player_indices;
                vec![a[2].clone(), a[3].clone()]
            }
            TournamentStage::Final => {
                let wa = self.winner_a.clone().unwrap_or_default();
                let wb = self.winner_b.clone().unwrap_or_default();
                vec![wa, wb]
            }
        }
    }

    /// 当前阶段的显示名称
    pub fn stage_label(&self) -> &str {
        match self.stage {
            TournamentStage::SemifinalA => "SemifinalA",
            TournamentStage::SemifinalB => "SemifinalB",
            TournamentStage::Final => "Final",
            TournamentStage::NotStarted | TournamentStage::Completed => "Idle",
        }
    }

    /// 构建 TournamentStageInfo 发送给客户端
    pub fn stage_info(&self) -> TournamentStageInfo {
        let a = &self.player_indices;
        TournamentStageInfo {
            stage: self.stage_label().to_string(),
            active_players: self.active_players(),
            bracket_a: (
                a.first().cloned().unwrap_or_default(),
                a.get(1).cloned().unwrap_or_default(),
            ),
            bracket_b: (
                a.get(2).cloned().unwrap_or_default(),
                a.get(3).cloned().unwrap_or_default(),
            ),
            winner_a: self.winner_a.clone(),
            winner_b: self.winner_b.clone(),
        }
    }
}

// ============================================================
// 游戏房间
// ============================================================
pub struct GameRoom {
    pub room_code: String,
    pub state: RoomState,
    pub players: HashMap<String, RoomPlayer>,
    pub mode: common::GameMode,             // Classic / Tournament
    pub host_username: String,              // 房主（第一个加入的玩家）
    pub game_state: Option<GameState>,
    pub tick_rate_ms: u64,
    pub map_width: i32,
    pub map_height: i32,
    pub tx: broadcast::Sender<ServerMessage>,
    pub tournament: Option<TournamentState>,
}

impl GameRoom {
    pub fn new(room_code: String) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            room_code,
            state: RoomState::Waiting,
            players: HashMap::new(),
            mode: common::GameMode::Classic,
            host_username: String::new(),
            game_state: None,
            tick_rate_ms: 200,
            map_width: 30,
            map_height: 30,
            tx,
            tournament: None,
        }
    }

    // ---------- 静态工具 ----------

    /// 生成 4 位大写字母数字房间码（碰撞概率极低）
    pub fn generate_room_code() -> String {
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect(); // 排除易混淆字符 0/O/1/I
        (0..4).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
    }

    // ---------- 房间信息 ----------

    pub fn to_room_info(&self) -> RoomInfo {
        RoomInfo {
            room_code: self.room_code.clone(),
            player_count: self.players.len(),
            max_players: MAX_PLAYERS,
            mode: self.mode.to_string(),
            host_username: self.host_username.clone(),
        }
    }

    // ---------- 玩家管理 ----------

    /// 添加玩家。Waiting 状态才允许加入；Playing 时拒绝。
    /// 返回 true 表示加入成功。
    pub fn add_player(&mut self, username: String, color: String, mode: common::GameMode) -> bool {
        // 游戏中禁止加入
        if self.state != RoomState::Waiting {
            return false;
        }
        if self.players.len() >= MAX_PLAYERS {
            return false;
        }
        let is_first = self.players.is_empty();
        if is_first {
            self.mode = mode;
            self.host_username = username.clone();
        }
        self.players.entry(username.clone()).or_insert(RoomPlayer {
            username,
            is_ready: false,
            color,
        });
        true
    }

    /// 移除玩家（仅在 Waiting 状态使用）。
    /// Playing 状态下请使用 `forfeit_player` 而非此方法。
    /// 返回 Some(new_host) 表示房主已转移。
    pub fn remove_player(&mut self, username: &str) -> Option<String> {
        self.players.remove(username);

        // 重置所有 ready 和锦标赛（防止残局卡死）
        for p in self.players.values_mut() {
            p.is_ready = false;
        }
        self.tournament = None;
        self.game_state = None;
        self.state = RoomState::Waiting;

        // 房主离开 → 转移给剩余第一个玩家
        let mut new_host = None;
        if self.host_username == username && !self.players.is_empty() {
            self.host_username = self.players.keys().next().unwrap().clone();
            new_host = Some(self.host_username.clone());
        }

        // 房间彻底空了 → 重置 mode 和 host
        if self.players.is_empty() {
            self.mode = common::GameMode::Classic;
            self.host_username = String::new();
        }

        new_host
    }

    /// 游戏内投降 / 断线：将玩家蛇标记为死亡，玩家保留在房间内观战。
    /// 返回 true 表示成功标记。
    pub fn forfeit_player(&mut self, username: &str) -> bool {
        if let Some(ref mut gs) = self.game_state {
            if let Some(snake) = gs.snakes.iter_mut().find(|s| s.username == username && s.is_alive) {
                snake.is_alive = false;
                snake.just_died = true;
                return true;
            }
        }
        false
    }

    pub fn ready_player(&mut self, username: &str) -> bool {
        if let Some(player) = self.players.get_mut(username) {
            player.is_ready = true;
        }
        !self.players.is_empty() && self.players.values().all(|p| p.is_ready)
    }

    /// 检查是否应启动游戏（锦标赛模式：4 人；经典模式：至少 1 人即可练习）
    pub fn can_start_game(&self) -> bool {
        if !self.players.values().all(|p| p.is_ready) {
            return false;
        }
        match self.mode {
            common::GameMode::Tournament => {
                // 锦标赛：必须满 4 人且全 ready
                self.players.len() == 4 && self.tournament.is_none()
            }
            common::GameMode::Classic => {
                // 经典模式：至少 1 人即可（单人练习 / 多人混战）
                !self.players.is_empty()
            }
        }
    }

    /// 检查用户名是否为房主
    pub fn is_host(&self, username: &str) -> bool {
        self.host_username == username
    }

    // ---------- 锦标赛生命周期 ----------

    /// 启动锦标赛：分配 bracket，进入半决赛 A
    pub fn start_tournament(&mut self) -> TournamentStageInfo {
        let mut player_order: Vec<String> = self.players.keys().cloned().collect();
        player_order.sort(); // 按用户名固定顺序

        let ts = TournamentState {
            stage: TournamentStage::SemifinalA,
            player_indices: player_order,
            winner_a: None,
            winner_b: None,
            champion: None,
        };
        let info = ts.stage_info();
        self.tournament = Some(ts);
        // 初始化半决赛 A 的游戏
        let active = self.tournament.as_ref().unwrap().active_players();
        self.init_game_with_players(&active);
        info
    }

    /// 推进锦标赛到下一阶段，返回新的 stage_info
    /// 调用时机：一局游戏结束（tick 返回 false）
    pub fn advance_tournament(&mut self) -> Option<TournamentStageInfo> {
        // 记录当前阶段的胜者（在可变借用 tournament 前先读取 game_state）
        let winner = self
            .game_state
            .as_ref()
            .and_then(|gs| gs.snakes.iter().find(|s| s.is_alive))
            .map(|s| s.username.clone());

        // 推进阶段
        let next_stage = {
            let ts = self.tournament.as_mut()?;
            match ts.stage {
                TournamentStage::SemifinalA => {
                    ts.winner_a = winner;
                    ts.stage = TournamentStage::SemifinalB;
                    Some(TournamentStage::SemifinalB)
                }
                TournamentStage::SemifinalB => {
                    ts.winner_b = winner;
                    ts.stage = TournamentStage::Final;
                    Some(TournamentStage::Final)
                }
                TournamentStage::Final => {
                    ts.champion = winner;
                    ts.stage = TournamentStage::Completed;
                    None
                }
                _ => None,
            }
        };

        // 释放 tournament 可变借用后，再初始化游戏
        match next_stage {
            Some(_) => {
                let active = self
                    .tournament
                    .as_ref()
                    .map(|t| t.active_players())
                    .unwrap_or_default();
                self.init_game_with_players(&active);
                self.tournament.as_ref().map(|t| t.stage_info())
            }
            None => None, // 锦标赛结束
        }
    }

    /// 锦标赛结束 → 构建最终结果
    pub fn tournament_result(&self) -> Option<TournamentResultInfo> {
        let ts = self.tournament.as_ref()?;
        if ts.stage != TournamentStage::Completed {
            return None;
        }
        let champion = ts.champion.clone().unwrap_or_default();
        let runner_up = ts
            .winner_a
            .clone()
            .filter(|w| *w != champion)
            .or_else(|| ts.winner_b.clone().filter(|w| *w != champion))
            .unwrap_or_default();

        let mut rankings = Vec::new();
        rankings.push((champion.clone(), 1));
        rankings.push((runner_up.clone(), 2));
        // 第 3、4 名从余下玩家中取出
        for p in &ts.player_indices {
            if *p != champion && *p != runner_up {
                rankings.push((p.clone(), rankings.len() + 1));
            }
        }

        Some(TournamentResultInfo {
            champion,
            runner_up,
            rankings,
        })
    }

    /// 重置所有玩家的准备状态
    pub fn reset_ready(&mut self) {
        for p in self.players.values_mut() {
            p.is_ready = false;
        }
    }

    /// 当前游戏是否属于锦标赛
    pub fn is_tournament_active(&self) -> bool {
        self.tournament
            .as_ref()
            .map(|t| t.stage != TournamentStage::NotStarted && t.stage != TournamentStage::Completed)
            .unwrap_or(false)
    }

    /// 检查某个玩家是否可以在当前阶段操控蛇
    pub fn can_player_move(&self, username: &str) -> bool {
        match &self.tournament {
            Some(ts) => ts.active_players().contains(&username.to_string()),
            None => true, // 经典模式：所有玩家可操作
        }
    }
}
