// esp32s3/src/protocol.h
// 消息协议层：构建 ClientMessage JSON / 解析 ServerMessage JSON
// 与 Rust common crate 的 #[serde(tag = "type", content = "payload")] 格式对齐
// V2: 支持多房间 / Forfeit / 食物类型

#ifndef PROTOCOL_H
#define PROTOCOL_H

#include <ArduinoJson.h>
#include "config.h"

// ============================================================
// 构建 ClientMessage（发送给服务端）
// ============================================================

/** 构建 CreateRoom 报文（创建房间并自动加入为房主） */
static String buildCreateRoom(const char* roomCode = "") {
    JsonDocument doc;
    doc["type"] = "CreateRoom";
    doc["payload"]["username"] = PLAYER_USERNAME;
    doc["payload"]["color"]    = PLAYER_COLOR;
    doc["payload"]["mode"]     = "Classic";
    if (strlen(roomCode) > 0) {
        doc["payload"]["room_code"] = roomCode;
    } else {
        doc["payload"]["room_code"] = nullptr;
    }

    String out;
    serializeJson(doc, out);
    return out;
}

/** 构建 JoinRoom 报文（加入指定房间） */
static String buildJoinRoom(const char* roomCode = "") {
    JsonDocument doc;
    doc["type"] = "JoinRoom";
    doc["payload"]["room_code"] = strlen(roomCode) > 0 ? roomCode : "";
    doc["payload"]["username"]  = PLAYER_USERNAME;
    doc["payload"]["color"]     = PLAYER_COLOR;
    doc["payload"]["mode"]      = "Classic";   // 默认经典模式；锦标赛改 "Tournament"

    String out;
    serializeJson(doc, out);
    return out;
}

/** 构建 ListRooms 报文（请求可加入的房间列表） */
static String buildListRooms() {
    JsonDocument doc;
    doc["type"] = "ListRooms";
    String out;
    serializeJson(doc, out);
    return out;
}

/** 构建 Ready 报文 */
static String buildReady() {
    JsonDocument doc;
    doc["type"] = "Ready";
    String out;
    serializeJson(doc, out);
    return out;
}

/** 构建 LeaveRoom 报文 */
static String buildLeaveRoom() {
    JsonDocument doc;
    doc["type"] = "LeaveRoom";
    String out;
    serializeJson(doc, out);
    return out;
}

/**
 * 构建 ChangeDirection 报文
 * @param dir  方向字符串："Up" | "Down" | "Left" | "Right"
 */
static String buildChangeDirection(const char* dir) {
    JsonDocument doc;
    doc["type"] = "ChangeDirection";
    doc["payload"]["direction"] = dir;
    String out;
    serializeJson(doc, out);
    return out;
}

/** 构建 Forfeit 报文（游戏内投降） */
static String buildForfeit() {
    JsonDocument doc;
    doc["type"] = "Forfeit";
    String out;
    serializeJson(doc, out);
    return out;
}

/** 构建 Ping 心跳报文 */
static String buildPing() {
    JsonDocument doc;
    doc["type"] = "Ping";
    String out;
    serializeJson(doc, out);
    return out;
}

// ============================================================
// 解析 ServerMessage（从服务端接收）
// ============================================================

/**
 * 消息回调接口
 * 当前仅打印到串口；后续加屏幕时可在此处渲染 UI
 */
static void onRoomStatus(JsonObject payload) {
    const char* code   = payload["room_code"]   | "?";
    const char* mode   = payload["mode"]        | "?";
    const char* host   = payload["host_username"] | "";
    int         tick   = payload["tick_rate_ms"]   | 0;
    JsonArray   plArr  = payload["players"].as<JsonArray>();

    Serial.printf("[RoomStatus] 房间=%s  模式=%s  房主=%s  速度=%dms  玩家数=%d\n",
                  code, mode, host, tick, plArr.size());
}

static void onRoomList(JsonArray rooms) {
    Serial.println("[RoomList] 可加入的房间:");
    for (JsonVariant item : rooms) {
        JsonObject r = item.as<JsonObject>();
        const char* code = r["room_code"] | "?";
        const char* mode = r["mode"] | "?";
        const char* host = r["host_username"] | "";
        int count = r["player_count"] | 0;
        int maxp  = r["max_players"] | 0;
        Serial.printf("  房间=%s  模式=%s  房主=%s  人数=%d/%d\n", code, mode, host, count, maxp);
    }
}

static void onGameStart() {
    Serial.println("[GameStart] 比赛开始！");
}

static void onGameFrame(JsonObject payload) {
    // 高频帧数据（30x30 网格），当前仅打印 tick 计数
    int tick = payload["tick_count"] | 0;
    JsonArray snakes = payload["snakes"].as<JsonArray>();
    JsonArray foods  = payload["foods"].as<JsonArray>();
    Serial.printf("[GameFrame] tick=%d 蛇数=%d 食物数=%d\n", tick, snakes.size(), foods.size());
}

static void onGameOver(JsonObject payload) {
    const char* winner = payload["winner_username"] | "无";
    Serial.printf("[GameOver] 胜者: %s\n", winner);

    JsonArray scores = payload["round_scores"].as<JsonArray>();
    for (JsonVariant item : scores) {
        JsonArray pair = item.as<JsonArray>();
        Serial.printf("  %s: +%d 分\n",
                      pair[0].as<const char*>(),
                      pair[1].as<int>());
    }
}

static void onTournamentStage(JsonObject payload) {
    const char* stage = payload["stage"] | "?";
    Serial.printf("[TournamentStage] 阶段=%s\n", stage);
}

static void onTournamentResult(JsonObject payload) {
    const char* champ = payload["champion"] | "?";
    Serial.printf("[TournamentResult] 冠军=%s\n", champ);
}

static void onError(JsonObject payload) {
    const char* msg = payload["message"] | "未知错误";
    Serial.printf("[ServerError] %s\n", msg);
}

/**
 * 解析单条 ServerMessage JSON 文本并分发到对应处理器
 * @param json 原始 JSON 字符串
 */
static void handleServerMessage(const String& json) {
    JsonDocument doc;
    DeserializationError err = deserializeJson(doc, json);
    if (err) {
        Serial.printf("[Protocol] JSON 解析失败: %s\n", err.c_str());
        return;
    }

    const char* type = doc["type"] | "";
    JsonObject payload = doc["payload"];

    if      (strcmp(type, "RoomStatus")       == 0) onRoomStatus(payload);
    else if (strcmp(type, "RoomList")         == 0) onRoomList(payload["rooms"].as<JsonArray>());
    else if (strcmp(type, "GameStart")        == 0) onGameStart();
    else if (strcmp(type, "GameFrame")        == 0) onGameFrame(payload);
    else if (strcmp(type, "GameOver")         == 0) onGameOver(payload);
    else if (strcmp(type, "TournamentStage")  == 0) onTournamentStage(payload);
    else if (strcmp(type, "TournamentResult") == 0) onTournamentResult(payload);
    else if (strcmp(type, "Error")            == 0) onError(payload);
    else if (strcmp(type, "Pong")             == 0) { /* quiet */ }
    else {
        Serial.printf("[Protocol] 未知消息类型: %s\n", type);
    }
}

#endif
