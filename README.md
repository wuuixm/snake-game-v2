# 🐍 Snake Arena — 多人联机贪吃蛇

<div align="center">

**一个跨平台的多人联机贪吃蛇游戏，支持 Web/桌面客户端 + 物理手柄操控**

</div>

---

## 系统架构总览

该系统采用 **C/S 架构（客户端-服务端）**，由四个端组成：

```
┌────────────────────────────────────────────────────────────────────────────────┐
│                                snake-game-server                               │
│                             Rust + Axum + WebSocket                            │
│                        （游戏逻辑 + 房间管理 + HTTP 鉴权）                     │
└───────┬────────────────────────────┬───────────────────────────┬───────────────┘
        │ WebSocket                  │ HTTP                      │ WebSocket
        ▼                            ▼                           ▼
┌───────────────┐          ┌──────────────────┐       ┌──────────────────────┐
│   桌面端      │          │   Web 浏览器     │       │  ESP32-S3 物理手柄   │
│  Tauri + Vue  │          │  Vue + Vite      │       │  Arduino + WiFi      │
│  (桌面应用)   │          │  (网页端)        │       │  (硬件控制器)        │
└───────────────┘          └──────────────────┘       └──────────────────────┘
                              │
                              │ HTTP (仅积分榜)
                              ▼
                    ┌──────────────────┐
                    │    PostgreSQL    │
                    │  snake_db        │
                    │  用户表 + 积分榜 │
                    └──────────────────┘
```

### 四端职责

| 端 | 技术栈 | 职责 |
|---|---|---|
| **common 公共层** | Rust (crate) | 定义所有端共享的数据结构（ClientMessage、ServerMessage、GameState、Direction 等），通过 `serde` 实现 JSON 序列化 |
| **snake-game-server** | Rust + Axum + WebSocket + PostgreSQL | 游戏核心服务：HTTP 鉴权、WebSocket 实时通信、房间管理、游戏逻辑（蛇移动、碰撞检测、食物生成）、锦标赛赛程编排 |
| **snake-game-client** | Vue 3 + Vite + Tailwind CSS v4 + Tauri 2 | 用户界面：鉴权登录、大厅/房间交互、Canvas 游戏渲染、实时积分榜、锦标赛晋级可视化；Tauri 封装为跨平台桌面应用 |
| **esp32s3** | Arduino + PlatformIO + WebSockets + ArduinoJson | 物理手柄固件：WiFi 联网、HTTP 鉴权、WebSocket 通信、按键扫描消抖、状态机编排、游戏方向操控 |

---

## 一、common — 公共数据契约层

> 路径：`common/src/lib.rs`

所有端共享的类型定义，通过 `serde` 标注 JSON 序列化格式，确保前端/后端/嵌入式设备之间数据格式严格一致。

### 枚举（Enum）

| 枚举 | 变体 | 说明 |
|---|---|---|
| **GameMode** | `Classic` / `Tournament` | 游戏模式：经典混战 / 四人锦标赛 |
| **FoodType** | `Apple` / `Cherry` / `Orange` / `Grape` / `Melon` / `SnakeRemains` | 6 种食物类型，各有不同外观与分值 |
| **Direction** | `Up` / `Down` / `Left` / `Right` | 四个移动方向 |
| **ClientMessage**（JSON tagged enum） | 见下方 | 客户端→服务端的全部报文类型 |
| **ServerMessage**（JSON tagged enum） | 见下方 | 服务端→客户端的全部报文类型 |

### 结构体（Struct）

| 结构体 | 字段 | 说明 |
|---|---|---|
| **Position** | `x: i32, y: i32` | 坐标点（网格单位） |
| **Snake** | `username, body, current_dir, is_alive, color, score, just_died` | 蛇的完整状态 |
| **FoodItem** | `position, food_type` | 食物实例 |
| **GameState** | `snakes, foods, tick_count` | 一帧完整的游戏状态快照 |
| **RoomPlayer** | `username, is_ready, color` | 房间内玩家信息 |
| **RoomInfo** | `room_code, player_count, max_players, mode, host_username` | 房间列表项 |
| **LeaderboardEntry** | `rank, username, max_score, total_games` | 全局积分榜条目 |
| **TournamentStageInfo** | `stage, active_players, bracket_a, bracket_b, winner_a, winner_b` | 锦标赛阶段信息 |
| **TournamentResultInfo** | `champion, runner_up, rankings` | 锦标赛最终排名 |

### 关键函数

| 函数 | 所属类型 | 说明 |
|---|---|---|
| `score_value()` | `FoodType` | 返回食物类型对应的分值：Apple=10, Cherry=20, Orange=15, Grape=25, Melon=50, SnakeRemains=15 |
| `as_str()` | `FoodType` | 返回食物类型的字符串标识（供客户端 SVG 图片映射） |
| `fmt()` | `GameMode` | 返回模式字符串："Classic" / "Tournament" |

### ClientMessage 报文（`#[serde(tag = "type", content = "payload")]`）

| type 字段 | payload | 说明 |
|---|---|---|
| `Register` | `{ username, password_hash }` | 账户注册（兼容，实际走 HTTP） |
| `Login` | `{ username, password_hash }` | 账户登录（兼容，实际走 HTTP） |
| `CreateRoom` | `{ room_code?, username, color, mode }` | 创建房间 |
| `ListRooms` | 空 | 请求房间列表 |
| `JoinRoom` | `{ room_code, username, color, mode }` | 加入房间 |
| `RefreshRoom` | 空 | 刷新房间状态 |
| `SetSpeed` | `{ tick_rate_ms }` | 调整游戏速度（房主专用） |
| `Ready` | 空 | 玩家准备 |
| `LeaveRoom` | 空 | 退出房间 |
| `ChangeDirection` | `{ direction }` | 改变蛇移动方向 |
| `Forfeit` | 空 | 游戏内投降 |
| `Ping` | 空 | 心跳 |

### ServerMessage 报文

| type 字段 | payload | 说明 |
|---|---|---|
| `AuthResult` | `{ success, message }` | 鉴权结果 |
| `RoomStatus` | `{ room_code, players, tick_rate_ms, is_all_ready, mode, host_username }` | 房间状态广播 |
| `RoomList` | `{ rooms }` | 房间列表 |
| `GameStart` | 空 | 游戏开始信号 |
| `GameFrame` | `GameState` | 实时帧数据 |
| `GameOver` | `{ winner_username, round_scores }` | 游戏结算 |
| `Leaderboard` | `[LeaderboardEntry]` | 全局积分榜（走 HTTP） |
| `TournamentStage` | `TournamentStageInfo` | 锦标赛阶段推进 |
| `TournamentResult` | `TournamentResultInfo` | 锦标赛最终结果 |
| `Error` | `{ message }` | 错误信息 |
| `Pong` | 空 | 心跳回复 |

---

## 二、snake-game-server — 游戏服务端

> 路径：`snake-game-server/`
> 技术栈：Rust + Axum 0.7 + WebSocket + sqlx + PostgreSQL + bcrypt
> 端口：8080（HTTP + WebSocket 共用）

### 模块结构

```
snake-game-server/src/
├── main.rs        # 入口：路由、CORS、数据库初始化、启动监听
├── room.rs        # 房间/Gameroom 数据结构、玩家管理、锦标赛赛程
├── game.rs        # 游戏核心逻辑：蛇移动、食物生成、碰撞检测
├── handlers.rs    # HTTP 请求处理器：注册、登录、积分榜
└── ws.rs          # WebSocket 连接管理、消息路由、游戏循环
```

---

### main.rs — 服务端入口

**作用**：初始化数据库连接池、配置 CORS 跨域、注册 HTTP 和 WebSocket 路由、启动 TCP 监听。

**定义的结构体**：

| 结构体 | 字段 | 说明 |
|---|---|---|
| `AppState` | `rooms, db_pool` | 全局共享状态，包含房间 Map 和 PG 连接池 |

**定义的函数**：

| 函数 | 说明 |
|---|---|
| `main()` | 1) 读取 `DATABASE_URL` 环境变量连接 PostgreSQL；2) 自动建表（`users` + `leaderboard`）；3) 创建 `Arc<AppState>`；4) 配置允许任意来源的 CORS；5) 注册 4 条路由（`/ws`, `/api/leaderboard`, `/api/auth/register`, `/api/auth/login`）；6) 绑定 `0.0.0.0:8080` 启动 |

---

### room.rs — 房间与锦标赛核心

**作用**：定义房间数据结构、玩家增删管理、准备机制、锦标赛生命周期（抽签→半决赛→决赛→冠军）。

**枚举**：

| 枚举 | 变体 | 说明 |
|---|---|---|
| `RoomState` | `Waiting` / `Playing` | 房间状态：等待中/游戏中 |
| `TournamentStage` | `NotStarted` / `SemifinalA` / `SemifinalB` / `Final` / `Completed` | 锦标赛五个阶段 |

**结构体**：

| 结构体 | 字段 | 说明 |
|---|---|---|
| `TournamentState` | `stage, player_indices, winner_a, winner_b, champion` | 锦标赛赛程状态 |
| `GameRoom` | `room_code, state, players, mode, host_username, game_state, tick_rate_ms, map_width, map_height, tx, tournament` | 游戏房间完整状态，含 broadcast channel 用于广播消息 |

**GameRoom 方法**：

| 方法 | 说明 |
|---|---|
| `new(room_code) → Self` | 构造函数，创建 broadcast channel（容量 100） |
| `generate_room_code() → String` | 生成 4 位大写字母数字码（排除 0/O/1/I 易混淆字符） |
| `to_room_info() → RoomInfo` | 转换为房间列表项 |
| `add_player(username, color, mode) → bool` | 添加玩家：仅 Waiting 状态允许，最多 4 人，首个为房主 |
| `remove_player(username) → Option<String>` | 移除玩家：房主转移、重置 ready/锦标赛/状态 |
| `forfeit_player(username) → bool` | 游戏内标记蛇死亡（掉线/投降） |
| `ready_player(username) → bool` | 标记玩家准备，返回是否全员 Ready |
| `can_start_game() → bool` | 检查是否可开始：全员 Ready + 经典模式≥1人/锦标赛4人 |
| `is_host(username) → bool` | 判断是否为房主 |
| `start_tournament() → TournamentStageInfo` | 启动锦标赛：排序玩家→分配 bracket→进入 SemifinalA |
| `advance_tournament() → Option<TournamentStageInfo>` | 推进锦标赛到下一阶段（SemifinalA→B→Final→Completed） |
| `tournament_result() → Option<TournamentResultInfo>` | 构建锦标赛最终排名（冠军→亚军→3/4名） |
| `reset_ready()` | 重置所有玩家的准备状态 |
| `is_tournament_active() → bool` | 判断当前是否在锦标赛进行中 |
| `can_player_move(username) → bool` | 锦标赛中仅活跃阶段玩家可操作；经典模式全部可操作 |

**TournamentState 方法**：

| 方法 | 说明 |
|---|---|
| `active_players() → Vec<String>` | 返回当前阶段允许操控的玩家列表 |
| `stage_label() → &str` | 返回阶段中文显示名 |
| `stage_info() → TournamentStageInfo` | 构建发送给客户端的阶段信息 |

---

### game.rs — 游戏核心逻辑

**作用**：实现贪吃蛇核心玩法——蛇移动、6 种食物生成、食物分值计算、三种碰撞检测（撞墙/撞自己/撞别人）、蛇死亡身体→食物转换、终局判定。

**GameRoom 方法（impl block）**：

| 方法 | 说明 |
|---|---|
| `init_game()` | 经典模式初始化：所有玩家参与，在中间位置分散生成蛇（间距8格），初始方向 Right，蛇身长度3 |
| `init_game_with_players(active_players)` | 锦标赛模式初始化：仅指定玩家生成蛇 |
| `spawn_foods(snakes, rng) → Vec<FoodItem>` | 初始批量生成 3~8 个食物，随机类型 |
| `spawn_single_food(snakes, existing, rng) → Option<FoodItem>` | 单个食物生成：避让所有蛇身和已有食物（最多尝试200次）；加权类型：Apple 30%, Cherry/Grape/Orange 各 20%, Melon 10% |
| `tick() → bool` | 游戏时钟单步推进：1) 蛇头前进插入；2) 食物判定（吃到→增长+加对应分数；未吃→去尾）；3) 碰撞检测（撞墙/撞自己/撞别的活蛇）；4) 死亡蛇身体→SnakeRemains 食物（蛇头除外）；5) 终局判定。返回 false 表示游戏结束 |

**碰撞检测规则**：
- 撞墙：`head.x < 0 || head.x >= map_width || head.y < 0 || head.y >= map_height`
- 撞自己：蛇头与 `body[1..]` 任意格子重合
- 撞别人：蛇头与其他活蛇的任意身体格子重合
- 终局：场上存活蛇数 ≤1 且总蛇数 > 1（或存活数为 0）

**食物分值表**：

| 食物类型 | 分值 | 生成概率 | 客户端 SVG |
|---|---|---|---|
| Apple | 10 分 | 30% | Apple.svg |
| Cherry | 20 分 | 20% | Cherry.svg |
| Orange | 15 分 | 20% | Orange.svg |
| Grape | 25 分 | 20% | Grape.svg |
| Melon | 50 分 | 10% | Melon.svg |
| SnakeRemains | 15 分 | 蛇死亡转化 | SnakeRemains.svg |

---

### handlers.rs — HTTP 请求处理器

**作用**：处理三个 RESTful API 端点，实现用户注册（bcrypt 加密）、登录（bcrypt 验证）、全局积分榜查询。

**结构体**：

| 结构体 | 字段 | 说明 |
|---|---|---|
| `AuthRequest` | `username, password_hash` | 注册/登录请求体 |
| `ApiResponse<T>` | `success, message, data` | 统一 HTTP 响应体 |

**函数**：

| 函数 | 路由 | 说明 |
|---|---|---|
| `register_handler(State, Json<AuthRequest>)` | `POST /api/auth/register` | 1) 验证用户名非空且密码≥6位；2) 服务端 bcrypt 哈希密码；3) 写入 users 表；4) 捕捉唯一约束冲突（用户名已存在） |
| `login_handler(State, Json<AuthRequest>)` | `POST /api/auth/login` | 1) 查询用户名；2) bcrypt 验证密码；3) 返回成功/失败/用户不存在 |
| `leaderboard_handler(State)` | `GET /api/leaderboard` | 1) 聚合查询 MAX(score)、COUNT(\*)；2) GROUP BY username；3) 按最高分降序取 TOP 10；4) 构造 LeaderboardEntry 列表 |

---

### ws.rs — WebSocket 处理器

**作用**：WebSocket 生命周期管理、17 种客户端消息路由、广播机制、游戏循环调度、掉线清理。

**函数**：

| 函数 | 说明 |
|---|---|
| `ws_handler(ws, State)` | WebSocket 升级路由入口 |
| `handle_socket(socket, state)` | 核心消息循环：接收 JSON→反序列化为 ClientMessage→match 分发。管理 `current_username` / `current_room_code` 连接上下文 |
| `build_room_status(room) → ServerMessage` | 构建 RoomStatus 消息辅助函数 |
| `spawn_game_loop(room_arc, db_pool, is_tournament)` | 游戏循环任务：1) 读取 `tick_rate_ms` 设置 interval；2) 循环调用 `room.tick()`；3) 每帧广播 `GameFrame`；4) 游戏结束写入积分榜；5) 经典模式直接结算；6) 锦标赛模式自动推进阶段（三段赛程完成后结算）；7) 最终广播 GameOver 并 reset_ready |

**消息路由表**（`match client_msg` 分支）：

| 消息类型 | 处理逻辑 |
|---|---|
| `CreateRoom` | 先退出旧房间→生成/使用房间码→创建 GameRoom→加入为房主→订阅广播→广播 RoomStatus |
| `ListRooms` | 遍历所有 Waiting 状态房间→返回 RoomList |
| `JoinRoom` | 先退出旧房间→验证房间存在→add_player→订阅广播→广播 RoomStatus |
| `RefreshRoom` | 点对点返回当前 RoomStatus |
| `SetSpeed` | 仅房主可调→更新 tick_rate_ms→广播 RoomStatus |
| `Ready` | 标记准备→广播 RoomStatus→check can_start_game→是则根据模式调 start_tournament 或 init_game→spawn_game_loop |
| `ChangeDirection` | 锦标赛检查活跃性→180度掉头防护→更新蛇方向 |
| `Forfeit` | 标记蛇死亡 |
| `LeaveRoom` | remove_player→房主转移广播→空房间自动删除→break 连接循环 |
| `Ping` | 回复 Pong |

**断线清理逻辑**（while 循环退出后）：
- Playing 状态掉线：`forfeit_player` 标记死亡，保留在房间
- Waiting 状态掉线：`remove_player` 完整移除，空房间自动删除

---

## 三、snake-game-client — 客户端

> 路径：`snake-game-client/`
> 技术栈：Vue 3 (Composition API) + Vite 6 + Tailwind CSS v4 + Tauri 2
> 开发命令：`pnpm dev`（Web 预览） / `pnpm tauri dev`（桌面应用）

### 架构分层

```
src/
├── main.js                          # Vue 应用入口
├── App.vue                          # 根组件：视图路由 + 全局 GameOverModal
├── style.css                        # Tailwind CSS v4 主题 + 自定义工具类 + 动画
├── network/
│   └── gameStream.js                # 纯网络传输层：WebSocket + HTTP
├── composables/
│   ├── useAuth.js                   # 鉴权模块
│   ├── useRoom.js                   # 房间管理模块
│   ├── useGame.js                   # 编排器（唯一状态 + 消息路由 + 跨域操作）
│   ├── useGameplay.js               # 游戏过程模块
│   └── useTournament.js             # 锦标赛模块
├── views/
│   ├── AuthView.vue                 # 鉴权视图
│   ├── LobbyView.vue                # 大厅视图
│   ├── RoomView.vue                 # 房间视图
│   └── GameView.vue                 # 游戏视图
└── components/
    ├── auth/
    │   ├── AuthForm.vue             # 登录/注册表单
    │   └── ServerAddressInput.vue   # 服务器地址输入
    ├── layout/
    │   ├── BgLayer.vue              # 全屏漂浮气泡背景
    │   └── GameHeader.vue           # 顶栏（Logo + 用户状态）
    ├── lobby/
    │   ├── PlayerConfigCard.vue     # 玩家昵称 + 颜色配置
    │   ├── RoomListCard.vue         # 可加入房间列表
    │   ├── CreateRoomCard.vue       # 创建新房间（选模式）
    │   └── LeaderboardCard.vue      # 全局积分榜
    ├── room/
    │   ├── RoomInfoBar.vue          # 房间信息 + 速度调节
    │   ├── CircularSpeedControl.vue # 圆环拖拽速度调节
    │   ├── PlayerListCard.vue       # 成员列表
    │   ├── RoomActionButtons.vue    # 准备/退出按钮
    │   └── TournamentBracket.vue    # 锦标赛晋级图
    └── game/
        ├── GameCanvas.vue           # Canvas 游戏渲染
        ├── GameTopBar.vue           # 游戏顶栏（已废弃，功能由 ArcadeDecor 承接）
        ├── ArcadeDecor.vue          # 游戏侧边装饰（摇杆 + 房间信息）
        ├── ScorePanel.vue           # 实时积分面板
        ├── GameBottomBar.vue        # 投降按钮
        └── GameOverModal.vue        # 结算弹窗
```

---

### main.js — 应用入口

**作用**：创建 Vue 应用实例并挂载到 `#app`。

| 函数 | 说明 |
|---|---|
| `createApp(App).mount("#app")` | 初始化 Vue 3 应用，引入全局样式 |

---

### App.vue — 根组件

**作用**：四视图路由（AuthView / LobbyView / RoomView / GameView）、全局背景层、游戏顶栏、通用结算弹窗。

**模板结构**：

| 组件/元素 | 条件 | 说明 |
|---|---|---|
| `<BgLayer />` | 始终显示 | 背景装饰 |
| `<GameHeader />` | 始终显示 | 顶栏 |
| `<AuthView />` | `!state.isLoggedIn` | 登录/注册 |
| `<LobbyView />` | `state.currentView === 'LOBBY'` | 大厅 |
| `<RoomView />` | `state.currentView === 'ROOM'` | 房间 |
| `<GameView />` | `state.currentView === 'PLAYING'` | 游戏中 |
| `<GameOverModal />` | `state.matchResult.show` | 结算弹窗 |

**导出的事件处理**：

| 函数 | 说明 |
|---|---|
| `handleLogin(username, password)` | 登录事件→alert 失败信息 |
| `handleRegister(username, password)` | 注册事件→alert 全部信息 |

---

### network/gameStream.js — 网络传输层

**作用**：纯网络层，不依赖 UI 状态。管理 WebSocket 生命周期、HTTP URL 构建、服务器地址持久化。

**导出的常量和函数**：

| 导出项 | 说明 |
|---|---|
| `getServerAddr() → String` | 从 localStorage 读取服务器地址，默认 `localhost:8080` |
| `setServerAddr(addr)` | 持久化服务器地址到 localStorage |
| `httpUrl(path) → String` | 拼接完整 HTTP URL：`http://{addr}{path}` |
| `connectWs() → Promise` | 建立 WebSocket 连接：已连接/连接中复用，支持错误重试 |
| `sendWs(type, payload?) → bool` | 发送消息：构建 `{ type, payload }` JSON 报文 |
| `disconnectWs()` | 主动断开 WebSocket |
| `isWsConnected() → bool` | 查询连接状态 |
| `onWsMessage(fn)` | 注册消息回调 |
| `onWsClose(fn)` | 注册关闭回调 |

---

### composables/useGame.js — 编排器（核心状态）

**作用**：持有唯一响应式 `state`，集成所有子 composable，集中处理 WebSocket 消息分发和跨域操作。

**`state` 响应式对象字段**：

| 字段 | 类型 | 来源模块 | 说明 |
|---|---|---|---|
| `isLoggedIn` | bool | useAuth | 登录状态 |
| `currentView` | string | 编排器 | 'LOGIN' / 'LOBBY' / 'ROOM' / 'PLAYING' |
| `username` | string | useAuth | 当前用户名 |
| `userColor` | string | useAuth | 蛇颜色 |
| `roomCode` | string | useRoom | 当前房间码 |
| `players` | array | useRoom | 房间玩家列表 |
| `tickRateMs` | number | useRoom | 游戏速度 |
| `isAllReady` | bool | useRoom | 全员准备 |
| `roomMode` | string | useRoom | 'Classic' / 'Tournament' |
| `hostUsername` | string | useRoom | 房主 |
| `roomList` | array | useRoom | 可加入房间 |
| `frameData` | object | useGameplay | 最新帧 |
| `matchResult` | object | useGameplay | 结算信息 |
| `tournament` | object | useTournament | 锦标赛状态 |

**WebSocket 消息路由表**（`onWsMessage`）：

| 消息类型 | 更新字段 |
|---|---|
| `RoomStatus` | `roomCode, players, tickRateMs, isAllReady, roomMode, hostUsername` |
| `RoomList` | `roomList` |
| `GameStart` | `currentView → 'PLAYING', matchResult.show = false` |
| `GameFrame` | `frameData` |
| `GameOver` | `matchResult.{winner, scores, show}` + `currentView → 'ROOM'` |
| `TournamentStage` | `tournament.{active, stage, activePlayers, bracketA, bracketB, winnerA, winnerB}` |
| `TournamentResult` | `tournament.{champion, runnerUp, rankings, active = false}` |
| `Error` | `alert()` |
| `Pong` | 忽略 |

**导出的跨域操作函数**：

| 函数 | 说明 |
|---|---|
| `logout()` | 断连→清所有状态→回登录页 |
| `leaveRoom()` | 发送 LeaveRoom→断连→清房间/游戏/锦标赛状态→回大厅 |
| `quitGame()` | 发送 Forfeit（蛇死亡观战，不退出房间） |
| `backToLobbyFromGameOver()` | 关闭弹窗→leaveRoom |
| `resetClient()` | 一键重置所有状态（保留登录）→回大厅 |

---

### composables/useAuth.js — 鉴权模块

**作用**：封装登录/注册的 HTTP 请求逻辑。

| 函数 | 说明 |
|---|---|
| `login(username, password)` → `{ success, message? }` | POST `/api/auth/login`，成功则更新 state 并切换到 LOBBY |
| `register(username, password)` → `{ success, message }` | POST `/api/auth/register` |
| `resetAuthState()` | 重置 isLoggedIn/username/userColor 为默认值 |

---

### composables/useRoom.js — 房间模块

**作用**：封装房间创建、列表查询、加入、准备、调速等 WebSocket 操作。

| 函数 | 说明 |
|---|---|
| `createRoom(roomCode?, mode?) → bool` | 连接 WS→发送 CreateRoom→切换到 ROOM 视图 |
| `listRooms()` | 连接 WS→发送 ListRooms |
| `joinRoom(roomCode, mode?) → bool` | 连接 WS→发送 JoinRoom→切换到 ROOM 视图 |
| `refreshRoom()` | 发送 RefreshRoom |
| `ready()` | 发送 Ready |
| `setSpeed(ms)` | 发送 SetSpeed |
| `isHost() → bool` | 判断当前用户是否为房主 |
| `tournamentWaiting() → bool` | 判断锦标赛模式是否等待凑齐 4 人 |
| `resetRoomState()` | 重置房间相关 state 字段 |

---

### composables/useGameplay.js — 游戏过程模块

**作用**：方向控制、投降、结算弹窗控制。

| 函数 | 说明 |
|---|---|
| `changeDirection(dir)` | 发送 ChangeDirection |
| `forfeit()` | 发送 Forfeit（蛇死亡，留在房间观战） |
| `closeGameOver()` | 关闭结算弹窗 |
| `resetGameplayState()` | 清空 frameData 和 matchResult |

---

### composables/useTournament.js — 锦标赛模块

**作用**：锦标赛阶段标签翻译、观众判断。

| 函数 | 说明 |
|---|---|
| `tournamentStageLabel() → string` | 返回阶段中文名称（SemifinalA→半决赛A组 等） |
| `isSpectator() → bool` | 锦标赛中当前用户是否为观众（非活跃玩家） |
| `resetTournamentState()` | 重置锦标赛 state 为默认值 |

---

### 视图组件（Views）

#### AuthView.vue — 鉴权视图

**作用**：服务器地址输入 + 登录/注册切换表单。

| 函数/属性 | 说明 |
|---|---|
| `serverAddr` (ref) | 服务器地址双向绑定 |
| `authMode` (ref) | "login" / "register" |
| `onAddrChange()` | 地址修改后持久化到 localStorage |
| `handleSubmit({ username, password })` | 根据 mode 发射 login/register 事件 |

#### LobbyView.vue — 大厅视图

**作用**：三栏布局——左栏（玩家配置+房间列表+创建房间）、右栏（全局积分榜）。

| 函数 | 说明 |
|---|---|
| `handleCreate(code, mode)` | 创建房间 |
| `handleReset()` | 一键重置客户端状态 |
| `fetchLeaderboard()` | GET `/api/leaderboard` 获取积分榜 |
| `onMounted` | 自动获取积分榜和房间列表 |

#### RoomView.vue — 房间视图

**作用**：五列 Grid——左 3/5（锦标赛晋级图/经典装饰）、右 2/5（成员列表+操作按钮）。

| 函数/事件 | 说明 |
|---|---|
| `handleRoomKeys(e)` | 键盘快捷键：Space=准备、Escape=离开 |
| `TournamentBracket` | 锦标赛模式显示晋级图 |
| `PlayerListCard` | 显示成员准备状态 |
| `RoomActionButtons` | 准备/退出按钮 |
| `CircularSpeedControl` | 房主拖拽调节速度 |

#### GameView.vue — 游戏视图

**作用**：三栏固定布局（14rem 摇杆装饰 + Canvas + 14rem 积分面板）。

| 计算属性/函数 | 说明 |
|---|---|
| `allSnakes` (computed) | 从 frameData 提取蛇列表（username, score, is_alive, color） |
| `handleForfeit()` | 投降标记 |
| `handleGameKeys(e)` | Escape=投降 |

---

### 组件详解（Components）

#### 布局组件

| 组件 | 文件 | 作用 |
|---|---|---|
| **BgLayer** | `layout/BgLayer.vue` | 全屏背景图 + 24 个浮动气泡动画（9大×15中），使用 CSS `float` 关键帧 |
| **GameHeader** | `layout/GameHeader.vue` | 顶栏：Logo "SnakeArena" + "联机贪吃蛇" + 用户状态显示/注销按钮 |

#### 鉴权组件

| 组件 | 文件 | 作用 |
|---|---|---|
| **AuthForm** | `auth/AuthForm.vue` | 登录/注册切换 Tabs + 表单（用户名+密码） |
| **ServerAddressInput** | `auth/ServerAddressInput.vue` | 服务器地址文本输入框 |

#### 大厅组件

| 组件 | 文件 | 作用 |
|---|---|---|
| **PlayerConfigCard** | `lobby/PlayerConfigCard.vue` | 玩家昵称展示 + 颜色选择器 |
| **RoomListCard** | `lobby/RoomListCard.vue` | 可加入房间列表（房间码+模式+房主+人数） + 重置/刷新按钮 |
| **CreateRoomCard** | `lobby/CreateRoomCard.vue` | 创建房间（自定义房间码 + 模式选择：经典/锦标赛） |
| **LeaderboardCard** | `lobby/LeaderboardCard.vue` | 全局积分榜 TOP 10 + 奖牌显示 + 当前用户高亮 |

#### 房间组件

| 组件 | 文件 | 作用 |
|---|---|---|
| **RoomInfoBar** | `room/RoomInfoBar.vue` | 房间信息（模式标签+房间码） + 速度调节区域（房主专属） |
| **CircularSpeedControl** | `room/CircularSpeedControl.vue` | SVG 圆环拖拽调节速度（50~500ms 步进 25ms） + 颜色渐变（快→慢 绿→黄→红） |
| **PlayerListCard** | `room/PlayerListCard.vue` | 玩家列表（用户名+身份标签+准备状态） |
| **RoomActionButtons** | `room/RoomActionButtons.vue` | 准备按钮（Space 快捷键） + 退出按钮（Esc 快捷键） |
| **TournamentBracket** | `room/TournamentBracket.vue` | 锦标赛晋级图：半决赛 A/B 组 → 箭头 → 总决赛，高亮当前阶段 |

#### 游戏组件

| 组件 | 文件 | 作用 |
|---|---|---|
| **GameCanvas** | `game/GameCanvas.vue` | Canvas 2D 渲染引擎：食物 SVG 图片渲染（6种）、蛇头（眼睛+高光）、蛇身（圆角方块+高光+尾部渐变缩小）、网格线装饰 |
| **ArcadeDecor** | `game/ArcadeDecor.vue` | 游戏侧边装饰：房间信息卡片 + 摇杆 SVG + AB 键 SVG + 观战/锦标赛标签 |
| **ScorePanel** | `game/ScorePanel.vue` | 实时积分面板：按分数降序排列、存活/阵亡状态、操作提示 |
| **GameBottomBar** | `game/GameBottomBar.vue` | 投降按钮（Esc 快捷键） + 操作提示（WASD/方向键） |
| **GameOverModal** | `game/GameOverModal.vue` | 结算弹窗（`<Teleport to="body">`）：获胜者展示、玩家分数列表、确认返回/回大厅按钮 |

### Canvas 渲染引擎（GameCanvas.vue 核心）

| 函数/变量 | 说明 |
|---|---|
| `FOOD_SVG_IMPORTS` | 6 种食物 SVG 图片导入映射 |
| `DIRECTION_CODE_MAP` | 键盘按键码→方向映射（WASD + 方向键） |
| `handleKeyDown(event)` | 键盘方向控制 |
| `drawGame(frame)` | 核心渲染函数：清屏→绘制网格线→6 种食物 SVG→蛇群渲染（蛇头：白色外圈+主色圆角+眼睛/瞳孔/高光；蛇身：圆角方块+高光+尾部渐变缩小） |
| `loadFoodImages()` | 预加载 6 种食物 SVG 图片 |
| `foodImages` | 缓存已加载的 Image 对象 |

---

### src-tauri — Tauri 桌面端封装

> 路径：`snake-game-client/src-tauri/`

| 文件 | 作用 |
|---|---|
| `src/main.rs` | Tauri 桌面应用入口，调用 lib.rs 的 `run()` |
| `src/lib.rs` | Tauri Builder 初始化，注册 opener 插件 + greet 命令 |
| `tauri.conf.json` | Tauri 配置：窗口标题/尺寸、构建命令、包标识符 |
| `Cargo.toml` | Rust 依赖：tauri, tauri-plugin-opener, serde, serde_json |
| `capabilities/default.json` | Tauri 2 权限配置 |
| `icons/` | 多平台应用图标（Windows/Mac/Linux/iOS/Android） |

---

## 四、esp32s3 — 物理手柄固件

> 路径：`esp32s3/`
> 技术栈：Arduino + PlatformIO + WebSockets + ArduinoJson
> 开发板：ESP32-S3-DevKitC-1
> 构建命令：`pio run` / `pio run -t upload`

### 固件架构

```
esp32s3/src/
├── main.cpp         # 主程序：状态机 + 按键扫描 + 网络编排
├── network.h        # 网络层：WiFi + HTTP 鉴权 + WebSocket
├── protocol.h       # 协议层：JSON 报文构建/解析
├── config.h         # 用户配置：WiFi/服务器/凭据/引脚（已 gitignored）
└── config.example.h # 配置模板（复制为 config.h 使用）
```

---

### main.cpp — 主控程序

**作用**：实现 GPIO 物理按键扫描（消抖）、状态机驱动、游戏操作映射。

**枚举**：

| 枚举 | 变体 | 说明 |
|---|---|---|
| `GamepadState` | `GP_LOBBY` / `GP_ROOM` / `GP_PLAYING` | 手柄状态机三态 |

**结构体**：

| 结构体 | 字段 | 说明 |
|---|---|---|
| `Button` | `pin, name, lastRaw, stable, lastChange` | 按键状态（消抖用） |

**定义的常量和变量**：

| 名称 | 值 | 说明 |
|---|---|---|
| `buttons[]` | 6 个 Button | SET/MID/UP/DOWN/LEFT/RIGHT |
| `BTN_COUNT` | 6 | 按键总数 |
| `gpState` | GP_LOBBY (初始) | 当前状态机状态 |
| `currentDir` | "Right" (初始) | 当前蛇方向（防 180° 急转） |
| `PING_INTERVAL_MS` | 30000 | 心跳间隔 30 秒 |

**函数**：

| 函数 | 说明 |
|---|---|
| `scanButtons() → Button*` | 100Hz 按键扫描：读取电平→比较变化→50ms 消抖→检测下降沿（HIGH→LOW 即按下） |
| `isValidDirection(newDir) → bool` | 方向防抖：禁止 180° 掉头（Up→Down / Down→Up / Left→Right / Right→Left） |
| `handleButton(Button&)` | 状态机主入口：根据 `gpState` 分发按键动作 |
| `onWsEventWithState(type, data, len)` | WebSocket 事件回调：解析消息类型→驱动状态机转换（LOBBY→ROOM→PLAYING→ROOM），同时调用 protocol 打印日志 |
| `setupWsCallback()` | 替换 network.h 默认回调为含状态机的版本 |
| `setup()` | 初始化：串口→按键 GPIO→WiFi→HTTP 鉴权→WebSocket→替换回调 |
| `loop()` | 主循环：wsLoop 心跳→30s Ping→按键扫描→handleButton→delay(5ms) |

**状态机转移表**：

| 当前状态 | 按键 | 动作 | 下一状态 |
|---|---|---|---|
| GP_LOBBY | MID | 发送 JoinRoom("1111") | —（等待 RoomStatus） |
| 收到 RoomStatus | — | — | GP_LOBBY → GP_ROOM |
| GP_ROOM | MID | 发送 Ready | —（等待 GameStart） |
| GP_ROOM | SET | 发送 LeaveRoom | GP_ROOM → GP_LOBBY |
| 收到 GameStart | — | 重置 currentDir | GP_ROOM → GP_PLAYING |
| GP_PLAYING | UP/DOWN/LEFT/RIGHT | 发送 ChangeDirection（校验合法方向） | — |
| GP_PLAYING | SET | 发送 Forfeit | — |
| 收到 GameOver | — | — | GP_PLAYING → GP_ROOM |

---

### network.h — 网络层

**作用**：WiFi 连接、HTTP 注册/登录、WebSocket 生命周期管理。

**全局变量**：

| 变量 | 类型 | 说明 |
|---|---|---|
| `ws` | `WebSocketsClient` | 全局 WebSocket 客户端实例 |

**函数**：

| 函数 | 说明 |
|---|---|
| `connectWiFi() → bool` | WiFi 连接（20 秒超时，失败则重启） |
| `httpRegister() → bool` | POST `/api/auth/register`：自动创建账号 |
| `httpLogin() → bool` | POST `/api/auth/login`：登录验证（失败 10 秒重试） |
| `authenticate() → bool` | 注册→登录两步走（失败 10 秒重试） |
| `onWsEvent(type, data, len)` | 默认 WebSocket 事件回调（打印消息到串口） |
| `connectWS() → bool` | 连接 WebSocket（5 秒握手超时）+ 注册回调 + 5 秒自动重连 |
| `wsSend(text)` | 发送 WebSocket 文本帧 |
| `wsLoop()` | 每轮 loop 调用的 WS 心跳 |
| `wsConnected() → bool` | 连接状态查询 |

---

### protocol.h — 协议层

**作用**：构建 ClientMessage JSON 报文、解析 ServerMessage JSON 报文。

**报文构建函数**：

| 函数 | 输出的 type | 说明 |
|---|---|---|
| `buildCreateRoom(roomCode?)` | `CreateRoom` | 创建房间（可选自定义码） |
| `buildJoinRoom(roomCode)` | `JoinRoom` | 加入指定房间 |
| `buildListRooms()` | `ListRooms` | 请求房间列表 |
| `buildReady()` | `Ready` | 准备 |
| `buildLeaveRoom()` | `LeaveRoom` | 退出房间 |
| `buildChangeDirection(dir)` | `ChangeDirection` | 方向控制 |
| `buildForfeit()` | `Forfeit` | 投降 |
| `buildPing()` | `Ping` | 心跳 |

**消息解析函数**：

| 函数 | 处理的 type | 说明 |
|---|---|---|
| `onRoomStatus(payload)` | `RoomStatus` | 打印房间信息到串口 |
| `onRoomList(rooms)` | `RoomList` | 列出可加入房间 |
| `onGameStart()` | `GameStart` | 打印开始信息 |
| `onGameFrame(payload)` | `GameFrame` | 打印 tick 计数、蛇数、食物数 |
| `onGameOver(payload)` | `GameOver` | 打印胜者和分数 |
| `onTournamentStage(payload)` | `TournamentStage` | 打印阶段名称 |
| `onTournamentResult(payload)` | `TournamentResult` | 打印冠军 |
| `onError(payload)` | `Error` | 打印错误信息 |
| `handleServerMessage(json)` | 所有类型 | 主解析器：反序列化 JSON→根据 type 分发到对应 handler |

---

### config.h — 硬件配置

**作用**：集中管理所有可配置参数。

**宏定义**：

| 宏 | 示例值 | 说明 |
|---|---|---|
| `WIFI_SSID` | `"<penguin>"` | WiFi 名称 |
| `WIFI_PASSWORD` | `"fsc000000"` | WiFi 密码 |
| `SERVER_HOST` | `"192.168.121.175"` | 服务端 IP |
| `SERVER_PORT` | `8080` | 服务端端口 |
| `WS_PATH` | `"/ws"` | WebSocket 路径 |
| `PLAYER_USERNAME` | `"esp32-1"` | 手柄玩家名 |
| `PLAYER_PASSWORD` | `"123456"` | 登录密码 |
| `PLAYER_COLOR` | `"#eb99f6"` | 蛇颜色 |
| `DEFAULT_ROOM_CODE` | `"1111"` | 默认加入房间码 |
| `BTN_SET/MID/UP/DOWN/LEFT/RIGHT` | 1/2/7/6/5/4 | GPIO 引脚号 |
| `DEBOUNCE_MS` | 50 | 按键消抖毫秒数 |

### platformio.ini — PlatformIO 构建配置

| 配置项 | 值 | 说明 |
|---|---|---|
| `platform` | `espressif32` | 乐鑫 ESP32 平台 |
| `board` | `esp32-s3-devkitc-1` | 开发板型号 |
| `framework` | `arduino` | Arduino 框架 |
| `monitor_speed` | 115200 | 串口监视器波特率 |
| `upload_port` | `/dev/ttyACM*` | 上传端口 |
| `lib_deps` | WebSockets 2.4.2 + ArduinoJson 7.x | 依赖库 |

---

## 五、数据库

### db.sql — 数据库初始化脚本

**数据库名**：`snake_db`

**用户**：`pgsql`

**表结构**：

#### `users` 用户表

| 列名 | 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `SERIAL` | PRIMARY KEY | 自增 ID |
| `username` | `VARCHAR(50)` | NOT NULL, UNIQUE | 用户名 |
| `password_hash` | `VARCHAR(255)` | NOT NULL | bcrypt 哈希后的密码 |
| `created_at` | `TIMESTAMPTZ` | DEFAULT NOW() | 创建时间 |

#### `leaderboard` 排行榜表

| 列名 | 类型 | 约束 | 说明 |
|---|---|---|---|
| `id` | `SERIAL` | PRIMARY KEY | 自增 ID |
| `username` | `VARCHAR(50)` | NOT NULL, FK→users(username) CASCADE | 玩家名 |
| `score` | `INTEGER` | DEFAULT 0, NOT NULL | 本局得分 |
| `played_at` | `TIMESTAMPTZ` | DEFAULT NOW() | 游戏时间 |

**积分榜查询逻辑**（见 `handlers.rs` `leaderboard_handler`）：

```sql
SELECT username, MAX(score) AS max_score, COUNT(*) AS total_games
FROM leaderboard
GROUP BY username
ORDER BY MAX(score) DESC
LIMIT 10;
```

---

## 六、项目构建配置

### 根目录 `Cargo.toml` — Rust 工作空间

```toml
[workspace]
members = [
    "common",
    "snake-game-server",
    "snake-game-client/src-tauri"
]
resolver = "2"
```

三个 Rust crate 组成工作空间：
- **common** — 公共数据契约库
- **snake-game-server** — 游戏服务端
- **snake-game-client/src-tauri** — 桌面客户端 Rust 后端（Tauri）

### `.env` — 环境变量

```env
DATABASE_URL=postgres://pgsql:123456@localhost:5432/snake_db
```

### 启动指南

```bash
# 1. 启动 PostgreSQL
# 2. 初始化数据库
z pgsql
docker compose start

# 3. 启动游戏服务端
cargo run -p snake-game-server

# 4a. 启动 Web 客户端
cd snake-game-client
pnpm dev

# 4b. 启动桌面客户端（可选）
pnpm tauri dev

# 5. 编译上传 ESP32 固件（可选）
cd esp32s3
pio run -t upload
```

---

## 七、通信协议流程图

```
┌──────────┐          ┌──────────────┐          ┌───────────┐
│ 客户端   │          │  GameServer  │          │  ESP32    │
│ (Vue)    │          │  (Rust)      │          │  (手柄)   │
└────┬─────┘          └──────┬───────┘          └────┬──────┘
     │                       │                       │
     │  POST /api/auth/      │                       │
     │  register/login       │                       │
     │──────────────────────>│                       │
     │  AuthResult           │                       │
     │<──────────────────────│                       │
     │                       │                       │
     │  WS: CreateRoom       │                       │
     │──────────────────────>│                       │
     │  WS: RoomStatus       │                       │
     │<──────────────────────│                       │
     │                       │                       │
     │  WS: ListRooms        │                       │
     │<─────── RoomList ─────│                       │
     │                       │                       │
     │  WS: JoinRoom         │                       │
     │──────────────────────>│                       │
     │  WS: RoomStatus       │                       │
     │<──────────────────────│                       │
     │                       │                       │
     │  WS: Ready            │                       │
     │──────────────────────>│                       │
     │  WS: GameStart        │                       │
     │<──────────────────────│                       │
     │                       │                       │
     │  WS: ChangeDirection  │   WS: ChangeDirection │
     │──────────────────────>│<──────────────────────│
     │                       │                       │
     │  WS: GameFrame(每帧)  │                       │
     │<──────────────────────│──────────────────────>│
     │                       │                       │
     │  WS: Forfeit          │                       │
     │──────────────────────>│                       │
     │                       │                       │
     │  WS: GameOver         │                       │
     │<──────────────────────│──────────────────────>│
     │                       │                       │
     │  GET /api/leaderboard │                       │
     │──────────────────────>│                       │
     │  Leaderboard (TOP 10) │                       │
     │<──────────────────────│                       │
```

---

## 八、游戏特色

1. **多端互联**：Web 浏览器、Tauri 桌面应用、ESP32 物理手柄可同时加入同一房间
2. **双模式**：经典混战（1~4 人自由练习）和四人锦标赛（半决赛→决赛，自动赛程编排）
3. **6 种食物系统**：Apple(10分)/Cherry(20分)/Orange(15分)/Grape(25分)/Melon(50分)/SnakeRemains(15分)，蛇死亡身体转化为食物
4. **实时 Canvas 渲染**：纯前端 2D Canvas 绘制，带眼睛/高光/尾部渐变的蛇 + 6 种 SVG 食物图片
5. **锦标赛晋级图可视化**：半决赛→总决赛双向箭头晋级图，当前阶段高亮
6. **圆环调速**：房主可通过拖拽圆环实时调整游戏速度（50ms~500ms/帧）
7. **物理手柄**：ESP32-S3 6 个物理按键 + WiFi 联网 + 50ms 消抖 + 180° 方向防转
8. **积分榜系统**：PostgreSQL 持久化每局成绩，TOP 10 全局排行

---

*项目基于 Rust + Vue 3 + Tauri + Axum + ESP32 全栈构建*
