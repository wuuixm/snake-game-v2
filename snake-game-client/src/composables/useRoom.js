// src/composables/useRoom.js
// 房间模块：创建/列表/加入/准备/调速/房主判断 + 房间状态重置

import { connectWs, sendWs } from '../network/gameStream';

export function useRoom(state) {

  /** 创建房间并自动加入为房主 */
  async function createRoom(roomCode, mode = 'Classic') {
    if (!state.username.trim()) return false;

    try {
      await connectWs();
    } catch {
      alert('无法连接游戏服务器，请确认服务端已启动');
      return false;
    }

    sendWs('CreateRoom', {
      room_code: roomCode || null,
      username: state.username,
      color: state.userColor,
      mode,
    });
    state.currentView = 'ROOM';
    return true;
  }

  /** 获取可加入的房间列表（大厅用） */
  async function listRooms() {
    try {
      await connectWs();
    } catch {
      // 静默失败 — 可能服务器未启动
      return;
    }
    sendWs('ListRooms');
  }

  /** 加入指定房间 */
  async function joinRoom(roomCode, mode = 'Classic') {
    if (!state.username.trim()) return false;

    try {
      await connectWs();
    } catch {
      alert('无法连接游戏服务器，请确认服务端已启动');
      return false;
    }

    sendWs('JoinRoom', {
      room_code: roomCode,
      username: state.username,
      color: state.userColor,
      mode,
    });
    state.currentView = 'ROOM';
    return true;
  }

  /** 请求刷新当前房间状态 */
  function refreshRoom() {
    sendWs('RefreshRoom');
  }

  /** 进入房间（保留兼容旧代码：仍是 JoinRoom，但不带 room_code 则会失败） */
  async function enterRoom(mode = 'Classic') {
    // 兼容旧的调用方式：如果没有指定房间码，列出房间让用户选
    // 实际上这个函数已废弃，保留以不破坏现有引用
    return joinRoom('8888', mode); // fallback
  }

  function ready() {
    sendWs('Ready');
  }

  function setSpeed(ms) {
    sendWs('SetSpeed', { tick_rate_ms: ms });
  }

  /** 当前用户是否为房主 */
  function isHost() {
    return state.hostUsername === state.username;
  }

  /** 锦标赛模式是否仍在等待凑齐 4 人 */
  function tournamentWaiting() {
    return state.roomMode === 'Tournament' && state.players.length < 4;
  }

  /** 重置房间相关状态 */
  function resetRoomState() {
    state.roomCode = '';
    state.players = [];
    state.tickRateMs = 200;
    state.isAllReady = false;
    state.roomMode = 'Classic';
    state.hostUsername = '';
    state.roomList = [];
  }

  return {
    createRoom,
    listRooms,
    joinRoom,
    refreshRoom,
    enterRoom,    // 保留兼容
    ready,
    setSpeed,
    isHost,
    tournamentWaiting,
    resetRoomState,
  };
}
