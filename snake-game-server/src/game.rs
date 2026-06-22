// snake-game-server/src/game.rs
// 游戏核心逻辑：蛇移动 / 多食物生成 / 食物类型分值 / 碰撞检测 / 死亡蛇→食物

use rand::Rng;
use common::{Direction, Position, GameState, Snake, FoodItem, FoodType};
use crate::room::{GameRoom, RoomState};
use std::collections::HashSet;

/// 食物生成配置
const MIN_FOODS: usize = 3;
const MAX_FOODS: usize = 8;

impl GameRoom {
    // ---------- 游戏初始化 ----------

    /// 经典模式：所有玩家参战
    pub fn init_game(&mut self) {
        self.state = RoomState::Playing;
        let players: Vec<String> = self.players.keys().cloned().collect();
        self.init_game_with_players(&players);
    }

    /// 锦标赛模式：只为指定玩家创建蛇
    pub fn init_game_with_players(&mut self, active_players: &[String]) {
        self.state = RoomState::Playing;
        let mut rng = rand::thread_rng();
        let mut snakes = Vec::new();

        for (i, username) in active_players.iter().enumerate() {
            let start_y = 10 + (i as i32 * 8); // 中间位置，互相隔开

            // 从 players HashMap 获取颜色
            let color = self
                .players
                .get(username)
                .map(|p| p.color.clone())
                .unwrap_or_else(|| "#10b981".to_string());

            snakes.push(Snake {
                username: username.clone(),
                body: vec![
                    Position { x: 5, y: start_y },
                    Position { x: 4, y: start_y },
                    Position { x: 3, y: start_y },
                ],
                current_dir: Direction::Right,
                is_alive: true,
                color,
                score: 0,
                just_died: false,
            });
        }

        let foods = self.spawn_foods(&snakes, &mut rng);

        self.game_state = Some(GameState {
            snakes,
            foods,
            tick_count: 0,
        });
    }

    // ---------- 食物生成 ----------

    /// 初始批量生成食物（3~8 个，随机类型）
    fn spawn_foods(&self, snakes: &[Snake], rng: &mut impl Rng) -> Vec<FoodItem> {
        let count = rng.gen_range(MIN_FOODS..=MAX_FOODS);
        let mut foods = Vec::new();
        for _ in 0..count {
            if let Some(food) = self.spawn_single_food(snakes, &foods, rng) {
                foods.push(food);
            }
        }
        foods
    }

    /// 生成单个食物（避让所有蛇身和已有食物，最多尝试 200 次）
    fn spawn_single_food(
        &self,
        snakes: &[Snake],
        existing_foods: &[FoodItem],
        rng: &mut impl Rng,
    ) -> Option<FoodItem> {
        let mut occupied = HashSet::new();
        for snake in snakes {
            for pos in &snake.body {
                occupied.insert((pos.x, pos.y));
            }
        }
        for food in existing_foods {
            occupied.insert((food.position.x, food.position.y));
        }

        // 随机食物类型（加权）
        let food_type = match rng.gen_range(0..100) {
            0..=9 => FoodType::Melon,      // 10% — 高分稀有
            10..=29 => FoodType::Grape,    // 20%
            30..=49 => FoodType::Cherry,   // 20%
            50..=69 => FoodType::Orange,   // 20%
            _ => FoodType::Apple,          // 30% — 最常见
        };

        for _ in 0..200 {
            let candidate = Position {
                x: rng.gen_range(1..self.map_width - 1),
                y: rng.gen_range(1..self.map_height - 1),
            };
            if !occupied.contains(&(candidate.x, candidate.y)) {
                return Some(FoodItem {
                    position: candidate,
                    food_type,
                });
            }
        }
        None // 地图太满，放弃生成
    }

    // ---------- 游戏时钟 ----------

    pub fn tick(&mut self) -> bool {
        if self.state != RoomState::Playing {
            return false;
        }

        let mut gs = match self.game_state.take() {
            Some(state) => state,
            None => return false,
        };

        gs.tick_count += 1;
        let mut rng = rand::thread_rng();

        // ===== 1. 前进：所有活蛇头部插入新位置 =====
        for snake in gs.snakes.iter_mut() {
            if !snake.is_alive {
                continue;
            }
            let head = snake.body[0];
            let next_head = match snake.current_dir {
                Direction::Up => Position { x: head.x, y: head.y - 1 },
                Direction::Down => Position { x: head.x, y: head.y + 1 },
                Direction::Left => Position { x: head.x - 1, y: head.y },
                Direction::Right => Position { x: head.x + 1, y: head.y },
            };
            snake.body.insert(0, next_head);
        }

        // ===== 2. 食物判定：蛇头匹配任意食物 → 吃下（不缩尾 = 增长） =====
        let food_positions: HashSet<(i32, i32)> = gs
            .foods
            .iter()
            .map(|f| (f.position.x, f.position.y))
            .collect();
        let mut eaten_positions: HashSet<(i32, i32)> = HashSet::new();

        for snake in gs.snakes.iter_mut() {
            if !snake.is_alive {
                continue;
            }
            let head = (snake.body[0].x, snake.body[0].y);
            if food_positions.contains(&head) {
                // 找到被吃的食物，累加分数
                if let Some(food) = gs
                    .foods
                    .iter()
                    .find(|f| f.position.x == head.0 && f.position.y == head.1)
                {
                    snake.score += food.food_type.score_value();
                }
                eaten_positions.insert(head);
                // 不 pop 尾部 → 蛇身 +1
            } else {
                snake.body.pop(); // 没吃到食物 → 正常前进（去尾）
            }
        }

        // 移除被吃食物 + 等量补充
        gs.foods
            .retain(|f| !eaten_positions.contains(&(f.position.x, f.position.y)));
        for _ in 0..eaten_positions.len() {
            if let Some(new_food) = self.spawn_single_food(&gs.snakes, &gs.foods, &mut rng) {
                gs.foods.push(new_food);
            }
        }

        // ===== 3. 碰撞检测 =====
        let snakes_snapshot = gs.snakes.clone();
        for snake in gs.snakes.iter_mut() {
            if !snake.is_alive {
                continue;
            }
            let head = snake.body[0];

            // 撞墙
            if head.x < 0 || head.x >= self.map_width || head.y < 0 || head.y >= self.map_height {
                snake.is_alive = false;
                snake.just_died = true;
                continue;
            }
            // 撞自己（跳过头部，检查 body[1..]）
            if snake.body[1..].iter().any(|pos| *pos == head) {
                snake.is_alive = false;
                snake.just_died = true;
                continue;
            }
            // 撞别人（仅检查活蛇身体）
            for other_snake in &snakes_snapshot {
                if other_snake.username == snake.username || !other_snake.is_alive {
                    continue;
                }
                if other_snake.body.iter().any(|pos| *pos == head) {
                    snake.is_alive = false;
                    snake.just_died = true;
                    break;
                }
            }
        }

        // ===== 4. 死亡蛇身体 → 食物（SnakeRemains） =====
        for snake in gs.snakes.iter_mut() {
            if snake.just_died {
                // 跳过蛇头，身体其余格子变为食物
                for pos in &snake.body[1..] {
                    gs.foods.push(FoodItem {
                        position: *pos,
                        food_type: FoodType::SnakeRemains,
                    });
                }
                snake.just_died = false;
            }
        }

        // ===== 5. 终局判定 =====
        let alive_count = gs.snakes.iter().filter(|s| s.is_alive).count();
        self.game_state = Some(gs);

        if alive_count == 0 {
            false
        } else if alive_count <= 1
            && self
                .game_state
                .as_ref()
                .map(|gs| gs.snakes.len())
                .unwrap_or(0)
                > 1
        {
            // 只剩 ≤1 条蛇存活 → 游戏结束（只要场上蛇数 > 1）
            false
        } else {
            true
        }
    }
}
