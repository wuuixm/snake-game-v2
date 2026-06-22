// esp32s3/src/network.h
// 网络层：WiFi 连接 / HTTP 登录 / WebSocket 生命周期

#ifndef NETWORK_H
#define NETWORK_H

#include <WiFi.h>
#include <HTTPClient.h>
#include <WebSocketsClient.h>
#include <ArduinoJson.h>
#include "config.h"
#include "protocol.h"

// ============================================================
// 全局 WebSocket 客户端实例
// ============================================================
static WebSocketsClient ws;

// ============================================================
// WiFi
// ============================================================

static bool connectWiFi() {
    WiFi.begin(WIFI_SSID, WIFI_PASSWORD);
    Serial.printf("[WiFi] 正在连接 %s", WIFI_SSID);

    int attempts = 0;
    while (WiFi.status() != WL_CONNECTED && attempts < 40) {   // 20 秒超时
        delay(500);
        Serial.print(".");
        attempts++;
    }

    if (WiFi.status() == WL_CONNECTED) {
        Serial.println();
        Serial.printf("[WiFi] ✅ 已连接  IP=%s\n", WiFi.localIP().toString().c_str());
        return true;
    }

    Serial.println("\n[WiFi] ❌ 连接超时，60 秒后重启…");
    delay(60000);
    ESP.restart();
    return false;   // unreachable，消除编译器警告
}

// ============================================================
// HTTP 鉴权（先注册再登录，与 Tauri 客户端行为一致）
// ============================================================

/** POST /api/auth/register — 首次启动自动创建账号 */
static bool httpRegister() {
    HTTPClient http;
    char url[128];
    snprintf(url, sizeof(url), "http://%s:%d/api/auth/register", SERVER_HOST, SERVER_PORT);
    http.begin(url);
    http.addHeader("Content-Type", "application/json");

    JsonDocument body;
    body["username"]      = PLAYER_USERNAME;
    body["password_hash"] = PLAYER_PASSWORD;   // 服务端会 bcrypt 哈希后存储
    String payload;
    serializeJson(body, payload);

    int code = http.POST(payload);
    String response = http.getString();
    http.end();

    Serial.printf("[HTTP] POST /api/auth/register → %d\n", code);
    if (code <= 0) return false;

    JsonDocument doc;
    deserializeJson(doc, response);
    bool ok = doc["success"] | false;
    const char* msg = doc["message"] | "";

    if (ok) {
        Serial.printf("[HTTP] ✅ 注册成功: %s\n", msg);
    } else {
        Serial.printf("[HTTP] 注册结果: %s (可能已存在，继续登录)\n", msg);
    }
    return true;   // 无论注册成功还是"已存在"，都继续
}

/** POST /api/auth/login */
static bool httpLogin() {
    HTTPClient http;
    char url[128];
    snprintf(url, sizeof(url), "http://%s:%d/api/auth/login", SERVER_HOST, SERVER_PORT);
    http.begin(url);
    http.addHeader("Content-Type", "application/json");

    JsonDocument body;
    body["username"]      = PLAYER_USERNAME;
    body["password_hash"] = PLAYER_PASSWORD;
    String payload;
    serializeJson(body, payload);

    int code = http.POST(payload);
    String response = http.getString();
    http.end();

    Serial.printf("[HTTP] POST /api/auth/login → %d\n", code);

    if (code <= 0) {
        Serial.println("[HTTP] ❌ 无法连接服务器，10 秒后重试…");
        delay(10000);
        return false;
    }

    JsonDocument doc;
    deserializeJson(doc, response);
    bool ok = doc["success"] | false;
    const char* msg = doc["message"] | "";

    if (ok) {
        Serial.printf("[HTTP] ✅ 登录成功: %s\n", msg);
        return true;
    }

    Serial.printf("[HTTP] ❌ 登录失败: %s\n", msg);
    Serial.println("[HTTP] 请检查 config.h 中的 PLAYER_USERNAME / PLAYER_PASSWORD");
    return false;
}

/** 注册 → 登录 两步走；网络不通或密码错误则 10 秒后重试 */
static bool authenticate() {
    // 1. 先尝试注册（首次启动自动创建账号；已存在则跳过）
    if (!httpRegister()) return false;

    // 2. 登录
    while (!httpLogin()) {
        delay(10000);
    }
    return true;
}

// ============================================================
// WebSocket 事件回调
// ============================================================

static void onWsEvent(WStype_t type, uint8_t* data, size_t len) {
    switch (type) {
        case WStype_CONNECTED:
            Serial.printf("[WS] ✅ 已连接 %s:%d%s\n", SERVER_HOST, SERVER_PORT, WS_PATH);
            break;

        case WStype_TEXT: {
            String msg((const char*)data, len);
            handleServerMessage(msg);    // → protocol.h
            break;
        }

        case WStype_DISCONNECTED:
            Serial.println("[WS] 🔌 已断开");
            break;

        case WStype_ERROR:
            Serial.printf("[WS] ❌ 错误: %s\n", data);
            break;

        default:
            break;
    }
}

// ============================================================
// WebSocket 连接/发送
// ============================================================

static bool connectWS() {
    ws.begin(SERVER_HOST, SERVER_PORT, WS_PATH);
    ws.onEvent(onWsEvent);
    ws.setReconnectInterval(5000);   // 断线自动重连，5 秒间隔

    // 等待握手完成（最多 5 秒）
    unsigned long deadline = millis() + 5000;
    while (!ws.isConnected() && millis() < deadline) {
        ws.loop();
        delay(10);
    }
    return ws.isConnected();
}

/** 发送文本帧 */
static inline void wsSend(const String& text) {
    String mut = text;          // sendTXT 要求非 const 引用
    ws.sendTXT(mut);
}

/** 每次 loop() 必须调用的心跳 */
static inline void wsLoop() {
    ws.loop();
}

/** 是否已连接 */
static inline bool wsConnected() {
    return ws.isConnected();
}

#endif
