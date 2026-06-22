// src/composables/useTournament.js
// 锦标赛模块：阶段标签 / 观众判断 / 锦标赛状态重置

export function useTournament(state) {

  function tournamentStageLabel() {
    const labels = {
      SemifinalA: '🏆 半决赛 A 组',
      SemifinalB: '🏆 半决赛 B 组',
      Final: '👑 总决赛',
    };
    return labels[state.tournament.stage] || '';
  }

  function isSpectator() {
    return (
      state.tournament.active &&
      !state.tournament.activePlayers.includes(state.username)
    );
  }

  /** 重置锦标赛相关状态 */
  function resetTournamentState() {
    state.tournament = {
      active: false,
      stage: '',
      activePlayers: [],
      bracketA: ['', ''],
      bracketB: ['', ''],
      winnerA: null,
      winnerB: null,
      champion: '',
      runnerUp: '',
      rankings: [],
    };
  }

  return { tournamentStageLabel, isSpectator, resetTournamentState };
}
