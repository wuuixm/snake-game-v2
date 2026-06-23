// src/composables/useGame.js
// 编排器：持有唯一 state，集成子 composable，WebSocket 消息路由，跨域操作

import { reactive } from "vue";
import {
  connectWs,
  sendWs,
  disconnectWs,
  onWsMessage,
  onWsClose,
} from "../network/gameStream";
import { useAuth } from "./useAuth";
import { useRoom } from "./useRoom";
import { useGameplay } from "./useGameplay";
import { useTournament } from "./useTournament";

/** 标记：当前是否为主动断连（防止旧 WS 的 onclose 在 reconnect 后冲掉 ROOM 视图） */
let _intentionalDisconnect = false;

// ============================================================
// 唯一状态（所有子 composable 共享同一份）
// ============================================================
const state = reactive({
  // --- 鉴权（useAuth） ---
  isLoggedIn: false,

  // --- 视图路由（编排器持有） ---
  currentView: "LOGIN", // 'LOGIN' | 'LOBBY' | 'ROOM' | 'PLAYING'

  // --- 用户（useAuth） ---
  username: "",
  userColor: "#10b981",

  // --- 房间（useRoom） ---
  roomCode: "",
  players: [],
  tickRateMs: 200,
  isAllReady: false,
  roomMode: "Classic", // "Classic" | "Tournament"
  hostUsername: "", // 房主用户名
  roomList: [], // 可加入的房间列表

  // --- 游戏（useGameplay） ---
  frameData: null,

  // --- 结算（useGameplay） ---
  matchResult: {
    show: false,
    winner: "",
    scores: [],
  },

  // --- 锦标赛（useTournament） ---
  tournament: {
    active: false,
    stage: "", // "SemifinalA" | "SemifinalB" | "Final"
    activePlayers: [],
    bracketA: ["", ""],
    bracketB: ["", ""],
    winnerA: null,
    winnerB: null,
    champion: "",
    runnerUp: "",
    rankings: [],
  },
});

// ============================================================
// 初始化子 composable（传入同一份 state）
// ============================================================
const { login, register, resetAuthState } = useAuth(state);

const {
  createRoom,
  listRooms,
  joinRoom,
  refreshRoom,
  enterRoom,
  ready,
  setSpeed,
  isHost,
  tournamentWaiting,
  resetRoomState,
} = useRoom(state);

const { changeDirection, forfeit, closeGameOver, resetGameplayState } =
  useGameplay(state);

const { tournamentStageLabel, isSpectator, resetTournamentState } =
  useTournament(state);

// ============================================================
// 全局状态重置
// ============================================================
function resetAllState() {
  resetAuthState();
  resetRoomState();
  resetGameplayState();
  resetTournamentState();
  state.currentView = "LOGIN";
}

// ============================================================
// WebSocket 事件 → 状态路由（仅此一处）
// ============================================================
onWsMessage((msg) => {
  switch (msg.type) {
    case "RoomStatus":
      state.roomCode = msg.payload.room_code;
      state.players = msg.payload.players;
      state.tickRateMs = msg.payload.tick_rate_ms;
      state.isAllReady = msg.payload.is_all_ready;
      state.roomMode = msg.payload.mode || "Classic";
      state.hostUsername = msg.payload.host_username || "";
      break;

    case "RoomList":
      state.roomList = msg.payload.rooms || [];
      break;

    case "GameStart":
      state.currentView = "PLAYING";
      state.matchResult.show = false;
      break;

    case "GameFrame":
      state.frameData = msg.payload;
      break;

    case "GameOver":
      state.matchResult.winner = msg.payload.winner_username || "无";
      state.matchResult.scores = msg.payload.round_scores;
      state.matchResult.show = true;
      // 自然结束 → 留在房间（连接未断，可再来一局）
      state.currentView = "ROOM";
      break;

    case "TournamentStage":
      state.tournament.active = true;
      state.tournament.stage = msg.payload.stage;
      state.tournament.activePlayers = msg.payload.active_players;
      state.tournament.bracketA = msg.payload.bracket_a;
      state.tournament.bracketB = msg.payload.bracket_b;
      state.tournament.winnerA = msg.payload.winner_a;
      state.tournament.winnerB = msg.payload.winner_b;
      break;

    case "TournamentResult":
      state.tournament.champion = msg.payload.champion;
      state.tournament.runnerUp = msg.payload.runner_up;
      state.tournament.rankings = msg.payload.rankings;
      state.tournament.active = false;
      break;

    case "Error":
      alert(`服务器: ${msg.payload.message}`);
      break;

    case "Pong":
      break;
  }
});

onWsClose(() => {
  if (_intentionalDisconnect) {
    // 主动断连（leaveRoom / logout）→ 不干预 currentView
    _intentionalDisconnect = false;
    return;
  }
  // 非预期断连（网络闪断、服务端重启等）→ 兜底回大厅
  if (state.currentView === "ROOM" || state.currentView === "PLAYING") {
    state.currentView = "LOBBY";
    resetRoomState();
    resetGameplayState();
    resetTournamentState();
  }
});

// ============================================================
// 跨域操作（涉及多个子 composable 的复合动作）
// ============================================================

/** 注销（断连 + 清所有状态 + 回登录页） */
function logout() {
  _intentionalDisconnect = true;
  disconnectWs();
  resetAllState();
  state.matchResult.show = false;
}

/** 离开房间（仅在 ROOM 视图可用 — 通知服务端 + 断开连接 + 回大厅） */
function leaveRoom() {
  sendWs("LeaveRoom");
  _intentionalDisconnect = true;
  disconnectWs();
  resetRoomState();
  resetGameplayState();
  resetTournamentState();
  state.currentView = "LOBBY";
}

/** 游戏内投降（不再退出房间，改为 Forfeit — 蛇死亡后留在房间观战） */
function quitGame() {
  forfeit();
}

/** 关闭结算弹窗，回大厅 */
function backToLobbyFromGameOver() {
  closeGameOver();
  leaveRoom();
}

/**
 * 一键重置：除登录状态外，清空所有房间/游戏/锦标赛/用户设置，回到 LOBBY。
 * 用于出现状态 bug 时手动恢复。
 */
function resetClient() {
  _intentionalDisconnect = true;
  sendWs("LeaveRoom");
  disconnectWs();
  resetRoomState();
  resetGameplayState();
  resetTournamentState();
  // 保留登录状态，重置其他用户设置
  state.userColor = "#10b981";
  state.currentView = "LOBBY";
  state.roomList = [];
}

// ============================================================
// 公开 composable（兼容旧组件调用方式）
// ============================================================
export function useGame() {
  return {
    state,
    // 鉴权
    login,
    register,
    logout,
    // 大厅 / 房间
    createRoom,
    listRooms,
    joinRoom,
    refreshRoom,
    enterRoom, // 保留兼容
    // 房间
    leaveRoom,
    ready,
    setSpeed,
    // 游戏
    changeDirection,
    forfeit,
    quitGame,
    // 结算
    closeGameOver,
    backToLobbyFromGameOver,
    // 锦标赛
    tournamentStageLabel,
    isSpectator,
    // 房间
    isHost,
    tournamentWaiting,
    // 一键重置
    resetClient,
  };
}
