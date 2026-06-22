// esp32s3/src/main.cpp
// 贪吃蛇 ESP32-S3 物理手柄固件
// ── 状态机 + 按键扫描 + 网络编排

#include <Arduino.h>
#include "config.h"
#include "protocol.h"
#include "network.h"

// ============================================================
// 状态机
// ============================================================
enum GamepadState {
    GP_LOBBY,    // 已登录，未入房（会自动发送 JoinRoom）
    GP_ROOM,     // 在房间中，可按 MID 准备
    GP_PLAYING,  // 游戏中，方向键操控蛇
};

static GamepadState gpState = GP_LOBBY;

// ============================================================
// 按键
// ============================================================
struct Button {
    uint8_t pin;
    const char* name;
    bool       lastRaw;       // 上一轮 digitalRead
    bool       stable;        // 消抖后的稳定值（LOW=按下）
    unsigned long lastChange; // 最后翻转时刻
};

static Button buttons[] = {
    { BTN_SET,   "SET",   HIGH, HIGH, 0 },
    { BTN_MID,   "MID",   HIGH, HIGH, 0 },
    { BTN_UP,    "UP",    HIGH, HIGH, 0 },
    { BTN_DOWN,  "DOWN",  HIGH, HIGH, 0 },
    { BTN_LEFT,  "LEFT",  HIGH, HIGH, 0 },
    { BTN_RIGHT, "RIGHT", HIGH, HIGH, 0 },
};
static const uint8_t BTN_COUNT = sizeof(buttons) / sizeof(buttons[0]);

/** 调用频率 ~100 Hz；返回本次扫描中「刚按下」的按钮指针，否则 nullptr */
static Button* scanButtons() {
    for (uint8_t i = 0; i < BTN_COUNT; i++) {
        Button& b = buttons[i];
        bool raw = digitalRead(b.pin);
        if (raw != b.lastRaw) {
            b.lastChange = millis();
        }
        b.lastRaw = raw;

        if (raw != b.stable && (millis() - b.lastChange) > DEBOUNCE_MS) {
            // 状态稳定翻转
            bool prev = b.stable;
            b.stable = raw;
            if (prev == HIGH && raw == LOW) {   // 下降沿 → 刚按下
                return &b;
            }
        }
    }
    return nullptr;
}

// ============================================================
// 方向防抖（防止 180° 急转弯，与服务端检查一致）
// ============================================================
static const char* currentDir = "Right";   // 服务器初始化时默认 Right

static bool isValidDirection(const char* newDir) {
    if (strcmp(currentDir, "Up")    == 0 && strcmp(newDir, "Down")  == 0) return false;
    if (strcmp(currentDir, "Down")  == 0 && strcmp(newDir, "Up")    == 0) return false;
    if (strcmp(currentDir, "Left")  == 0 && strcmp(newDir, "Right") == 0) return false;
    if (strcmp(currentDir, "Right") == 0 && strcmp(newDir, "Left")  == 0) return false;
    return true;
}

// ============================================================
// 状态机入口：根据当前状态 + 按键分发动作
// ============================================================
static void handleButton(Button& b) {
    Serial.printf("[BTN] %s (state=%d)\n", b.name, gpState);

    switch (gpState) {

        // -------- LOBBY：等待自动入房 --------
        case GP_LOBBY:
            if (strcmp(b.name, "MID") == 0) {
                // 手动触发 JoinRoom（加入固定房间 "1111"）
                wsSend(buildJoinRoom(DEFAULT_ROOM_CODE));
                Serial.printf("[Action] JoinRoom(%s) → 等待 RoomStatus...\n", DEFAULT_ROOM_CODE);
            }
            break;

        // -------- ROOM：可准备 / 退出 --------
        case GP_ROOM:
            if (strcmp(b.name, "MID") == 0) {
                wsSend(buildReady());
                Serial.println("[Action] Ready → 等待 GameStart...");
            } else if (strcmp(b.name, "SET") == 0) {
                wsSend(buildLeaveRoom());
                gpState = GP_LOBBY;
                Serial.println("[Action] LeaveRoom → 回到 LOBBY");
            }
            break;

        // -------- PLAYING：方向 + 投降 --------
        case GP_PLAYING:
            if (strcmp(b.name, "SET") == 0) {
                wsSend(buildForfeit());
                Serial.println("[Action] Forfeit → 蛇死亡，观战中...");
            } else if (strcmp(b.name, "UP") == 0 && isValidDirection("Up")) {
                wsSend(buildChangeDirection("Up"));
                currentDir = "Up";
            } else if (strcmp(b.name, "DOWN") == 0 && isValidDirection("Down")) {
                wsSend(buildChangeDirection("Down"));
                currentDir = "Down";
            } else if (strcmp(b.name, "LEFT") == 0 && isValidDirection("Left")) {
                wsSend(buildChangeDirection("Left"));
                currentDir = "Left";
            } else if (strcmp(b.name, "RIGHT") == 0 && isValidDirection("Right")) {
                wsSend(buildChangeDirection("Right"));
                currentDir = "Right";
            }
            break;
    }
}

// ============================================================
// 在 network.h 的 connectWS() 中注册了默认回调 onWsEvent。
// connectWS() 之后调用 setupWsCallback() 替换为含状态机逻辑的版本。
// ============================================================

static void onWsEventWithState(WStype_t type, uint8_t* data, size_t len) {
    switch (type) {
        case WStype_CONNECTED:
            Serial.printf("[WS] ✅ 已连接 %s:%d%s\n", SERVER_HOST, SERVER_PORT, WS_PATH);
            // 不自动加入房间 — 按下 MID 键时才会 JoinRoom("1111")
            // 房间由 Tauri 客户端创建，ESP32 仅作为控制器加入
            Serial.println("[Auto] 等待 MID 按键加入房间...");
            break;

        case WStype_TEXT: {
            String msg((const char*)data, len);

            // 在 protocol 解析之前，先提取 type 字段以驱动状态机
            JsonDocument doc;
            if (!deserializeJson(doc, msg)) {
                const char* msgType = doc["type"] | "";
                if (strcmp(msgType, "RoomStatus") == 0) {
                    if (gpState == GP_LOBBY) {
                        gpState = GP_ROOM;
                        Serial.println("[State] LOBBY → ROOM");
                    }
                } else if (strcmp(msgType, "GameStart") == 0) {
                    gpState = GP_PLAYING;
                    currentDir = "Right";
                    Serial.println("[State] ROOM → PLAYING");
                } else if (strcmp(msgType, "GameOver") == 0) {
                    gpState = GP_ROOM;
                    Serial.println("[State] PLAYING → ROOM");
                }
            }

            // 转发给 protocol 解析器（打印日志）
            handleServerMessage(msg);
            break;
        }

        case WStype_DISCONNECTED:
            Serial.println("[WS] 🔌 已断开");
            gpState = GP_LOBBY;
            break;

        case WStype_ERROR:
            Serial.printf("[WS] ❌ 错误: %s\n", data);
            break;

        default:
            break;
    }
}

// network.h 中的 ws 已通过 onEvent 注册 onWsEvent。
// 需要在 connectWS() 之后替换回调：
static void setupWsCallback() {
    ws.onEvent(onWsEventWithState);
}

// ============================================================
// setup / loop
// ============================================================
void setup() {
    Serial.begin(115200);
    delay(500);
    Serial.println("\n\n╔══════════════════════════════════╗");
    Serial.println("║  🐍 Snake Gamepad — ESP32-S3   ║");
    Serial.println("╚══════════════════════════════════╝");

    // --- 按键初始化 ---
    for (uint8_t i = 0; i < BTN_COUNT; i++) {
        pinMode(buttons[i].pin, INPUT_PULLUP);
    }

    // --- WiFi ---
    connectWiFi();

    // --- 鉴权（先注册再登录，与 Tauri 客户端一致） ---
    Serial.println("[Init] 正在鉴权...");
    while (!authenticate()) {
        delay(10000);   // authenticate 内部有详细的重试日志
    }

    // --- WebSocket ---
    Serial.println("[Init] 正在连接 WebSocket...");
    if (connectWS()) {
        setupWsCallback();   // 替换回调为含状态机的版本
    } else {
        Serial.println("[Init] ❌ WebSocket 握手失败，60 秒后重启");
        delay(60000);
        ESP.restart();
    }

    Serial.println("[Init] ✅ 手柄就绪\n");
}

// 心跳间隔
static unsigned long lastPingMs = 0;
#define PING_INTERVAL_MS 30000

void loop() {
    wsLoop();   // WebSocket 收/发/重连

    // 定期心跳
    if (millis() - lastPingMs > PING_INTERVAL_MS) {
        lastPingMs = millis();
        wsSend(buildPing());
    }

    // 扫描按键
    Button* pressed = scanButtons();
    if (pressed) {
        handleButton(*pressed);
    }

    delay(5);   // 约 200 Hz 轮询，按键消抖窗口 50ms 足够覆盖
}
