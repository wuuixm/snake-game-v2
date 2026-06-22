// esp32s3/src/config.example.h
// 配置模板——部署时复制为 config.h 并填入真实值
//
//   cp esp32s3/src/config.example.h esp32s3/src/config.h
//
// config.h 已在 .gitignore 中排除，不会被提交到仓库。

#ifndef CONFIG_H
#define CONFIG_H

// ============================================================
// WiFi
// ============================================================
#define WIFI_SSID     "你的WiFi名"
#define WIFI_PASSWORD "你的WiFi密码"

// ============================================================
// 贪吃蛇服务端地址
// ============================================================
#define SERVER_HOST   "192.168.1.100"
#define SERVER_PORT   8080
#define WS_PATH       "/ws"

// ============================================================
// 玩家身份（固件首次启动时自动注册，无需手动操作）
// ============================================================
#define PLAYER_USERNAME "esp32"
#define PLAYER_PASSWORD "123456"
#define PLAYER_COLOR    "#ff5722"

// ============================================================
// 按键引脚
// ============================================================
#define BTN_SET     1
#define BTN_MID     2
#define BTN_UP      7
#define BTN_DOWN    6
#define BTN_LEFT    5
#define BTN_RIGHT   4

#define DEBOUNCE_MS 50

#endif
