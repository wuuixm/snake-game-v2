// src/composables/useGameplay.js
// 游戏过程模块：方向控制 / 投降 / 结算弹窗 / 游戏相关状态重置

import { sendWs } from '../network/gameStream';

export function useGameplay(state) {

  function changeDirection(dir) {
    sendWs('ChangeDirection', { direction: dir });
  }

  /** 游戏内投降（蛇死亡，留在房间观战直到对局结束） */
  function forfeit() {
    sendWs('Forfeit');
  }

  /** 关闭结算弹窗，留在房间（可再次准备） */
  function closeGameOver() {
    state.matchResult.show = false;
  }

  /** 重置游戏相关状态（帧数据 + 结算） */
  function resetGameplayState() {
    state.frameData = null;
    state.matchResult = { show: false, winner: '', scores: [] };
  }

  return { changeDirection, forfeit, closeGameOver, resetGameplayState };
}
