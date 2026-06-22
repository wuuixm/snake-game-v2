// src/network/gameStream.js
// 纯网络传输层：只负责 WebSocket 生命周期和原始消息收发。
// 完全不依赖、不修改任何 UI 状态。

// ---------- 服务器地址配置 ----------
const STORAGE_KEY = 'snake_server_addr';
const DEFAULT_ADDR = 'localhost:8080';

export function getServerAddr() {
  try {
    return localStorage.getItem(STORAGE_KEY) || DEFAULT_ADDR;
  } catch {
    return DEFAULT_ADDR;
  }
}

export function setServerAddr(addr) {
  try {
    localStorage.setItem(STORAGE_KEY, addr);
  } catch { /* noop */ }
}

function wsUrl() {
  return `ws://${getServerAddr()}/ws`;
}

export function httpUrl(path) {
  return `http://${getServerAddr()}${path}`;
}

let ws = null;
let connectPromise = null;
let onMessageHandler = null;
let onCloseHandler = null;

// ---------- 公开 API ----------

/** 建立 WebSocket 连接，可安全重复调用（已连接/连接中会复用） */
export function connectWs() {
  // 已连接 → 立即 resolve
  if (ws && ws.readyState === WebSocket.OPEN) {
    return Promise.resolve();
  }

  // 正在连接中 → 复用同一个 promise
  if (connectPromise) {
    return connectPromise;
  }

  // 清理旧的已关闭 socket
  if (ws) {
    ws.close();
    ws = null;
  }

  connectPromise = new Promise((resolve, reject) => {
    ws = new WebSocket(wsUrl());

    ws.onopen = () => {
      console.log('🌐 [WS] 连接已建立');
      connectPromise = null;
      resolve();
    };

    ws.onerror = () => {
      connectPromise = null;
      reject(new Error('WebSocket 连接失败'));
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (onMessageHandler) onMessageHandler(msg);
      } catch (e) {
        console.error('[WS] 报文解析失败:', e);
      }
    };

    ws.onclose = () => {
      console.log('🔌 [WS] 连接已关闭');
      connectPromise = null;
      if (onCloseHandler) onCloseHandler();
      ws = null;
    };
  });

  return connectPromise;
}

/** 发送消息，返回 true 表示已发送 */
export function sendWs(type, payload = {}) {
  if (!ws || ws.readyState !== WebSocket.OPEN) return false;
  const hasPayload = Object.keys(payload).length > 0;
  const message = hasPayload ? { type, payload } : { type };
  ws.send(JSON.stringify(message));
  return true;
}

/** 主动断开 */
export function disconnectWs() {
  if (ws) {
    ws.close();
    ws = null;
  }
  connectPromise = null;
}

/** 查询连接状态 */
export function isWsConnected() {
  return ws?.readyState === WebSocket.OPEN;
}

/** 注册消息回调（由上层状态机调用） */
export function onWsMessage(fn) {
  onMessageHandler = fn;
}

/** 注册关闭回调（由上层状态机调用） */
export function onWsClose(fn) {
  onCloseHandler = fn;
}
