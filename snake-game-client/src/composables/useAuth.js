// src/composables/useAuth.js
// 鉴权模块：登录 / 注册 / 注销 + 状态重置

import { httpUrl } from '../network/gameStream';

export function useAuth(state) {

  async function login(username, password) {
    try {
      const res = await fetch(httpUrl('/api/auth/login'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password_hash: password }),
      });
      const json = await res.json();
      if (json.success) {
        state.username = username;
        state.isLoggedIn = true;
        state.currentView = 'LOBBY';
        return { success: true };
      }
      return { success: false, message: json.message };
    } catch (err) {
      return { success: false, message: '网络连接失败: ' + err.message };
    }
  }

  async function register(username, password) {
    try {
      const res = await fetch(httpUrl('/api/auth/register'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password_hash: password }),
      });
      const json = await res.json();
      if (json.success) {
        return { success: true, message: '注册成功！请切换到登录界面。' };
      }
      return { success: false, message: json.message };
    } catch (err) {
      return { success: false, message: '网络连接失败: ' + err.message };
    }
  }

  /** 重置鉴权相关状态 */
  function resetAuthState() {
    state.isLoggedIn = false;
    state.username = '';
    state.userColor = '#10b981';
  }

  return { login, register, resetAuthState };
}
