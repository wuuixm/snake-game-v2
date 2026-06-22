<template>
  <div class="flex flex-col items-center justify-center min-h-screen bg-slate-900 p-4 font-mono text-slate-100">
    
    <div v-if="gameStatus === 'IDLE' || gameStatus === 'WAITING'" class="w-full max-w-md bg-slate-800 p-6 rounded-lg shadow-xl border border-slate-700">
      <h2 class="text-2xl font-bold mb-4 text-center text-emerald-400">🐍 多人联机贪吃蛇大厅</h2>
      
      <div v-if="gameStatus === 'IDLE'" class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1 text-slate-400">请输入玩家昵称：</label>
          <input v-model="username" type="text" class="w-full px-3 py-2 bg-slate-900 border border-slate-600 rounded focus:outline-none focus:border-emerald-500 text-slate-100" placeholder="例如: wuuixm" />
        </div>
        <button @click="connectAndJoin" class="w-full py-2 bg-emerald-600 hover:bg-emerald-500 rounded font-bold transition">
          进入默认房间 (8888)
        </button>
      </div>

      <div v-if="gameStatus === 'WAITING'" class="space-y-4">
        <div class="p-3 bg-slate-900 rounded border border-slate-700">
          <p class="text-sm text-slate-400">房间号: <span class="text-emerald-400 font-bold">{{ roomCode }}</span></p>
          <p class="text-sm text-slate-400 mt-2">当前玩家列表：</p>
          <ul class="list-disc list-inside mt-1 space-y-1">
            <li v-for="p in players" :key="p.username" :class="p.is_ready ? 'text-green-400' : 'text-amber-400'">
              {{ p.username }} —— {{ p.is_ready ? '已准备' : '未准备' }}
            </li>
          </ul>
        </div>

        <div class="flex gap-4">
          <button @click="sendReady" class="flex-1 py-2 bg-blue-600 hover:bg-blue-500 rounded font-bold transition">
            准备就绪 (Space)
          </button>
          <button @click="leaveRoom" class="flex-1 py-2 bg-rose-600 hover:bg-rose-500 rounded font-bold transition">
            退出房间 (Esc)
          </button>
        </div>
      </div>
    </div>

    <div v-if="gameStatus === 'PLAYING'" class="flex flex-col items-center space-y-4">
      <div class="flex justify-between w-full max-w-xl text-sm bg-slate-800 px-4 py-2 rounded border border-slate-700">
        <span class="text-blue-400">帧率时钟: {{ tickRate }}ms</span>
        <span class="text-emerald-400">生存战进行中...</span>
      </div>
      
      <canvas ref="gameCanvas" width="600" height="600" class="bg-slate-950 border-2 border-slate-700 rounded-lg shadow-2xl"></canvas>
      
      <p class="text-xs text-slate-500">操作提示: W/A/S/D 控制方向 | Esc 强退</p>
    </div>

    <div v-if="showResult" class="fixed inset-0 bg-black/80 flex items-center justify-center p-4">
      <div class="bg-slate-800 p-6 rounded-lg max-w-sm w-full border border-slate-700 text-center">
        <h3 class="text-xl font-bold text-amber-400 mb-2">🏆 本局结算</h3>
        <p class="text-sm text-slate-400 mb-4">获胜者: <span class="text-white font-bold">{{ winner }}</span></p>
        <div class="bg-slate-900 p-3 rounded text-left space-y-1 mb-4 text-sm">
          <div v-for="score in roundScores" :key="score[0]" class="flex justify-between">
            <span>{{ score[0] }}</span>
            <span class="text-amber-400">{{ score[1] }} 分</span>
          </div>
        </div>
        <button @click="showResult = false; gameStatus = 'WAITING'" class="w-full py-2 bg-emerald-600 hover:bg-emerald-500 rounded font-bold">
          返回房间大厅
        </button>
      </div>
    </div>

  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue';

// --- 状态变量 ---
const username = ref('');
const roomCode = ref('');
const players = ref([]);
const tickRate = ref(200);
const gameStatus = ref('IDLE'); // IDLE, WAITING, PLAYING
const showResult = ref(false);
const winner = ref('');
const roundScores = ref([]);

const gameCanvas = ref(null);
let ws = null;

// --- 键位映射解耦配置 (随时可以修改这里的字母来改键) ---
const KEY_MAP = {
  // 游戏控制快捷键
  ' ': 'READY',        // 空格键准备
  'Escape': 'LEAVE',   // Esc键退出
  
  // 方向盘控制键位 (WASD)
  'w': 'Up',
  's': 'Down',
  'a': 'Left',
  'd': 'Right',
  'W': 'Up', 'S': 'Down', 'A': 'Left', 'D': 'Right' // 兼容大写
};

// --- 网络通信核心 ---
const connectAndJoin = () => {
  if (!username.value.trim()) return alert('请输入昵称！');

  // 连接到我们已经全链路跑通的 Axum 服务端
  ws = new WebSocket('ws://localhost:8080/ws');

  // ws.onopen = () => {
  //   console.log('✅ 成功建立长连接');
  //   // 发送符合 lib.rs 契约的内部标签化 JSON
  //   ws.send(JSON.stringify({
  //     type: 'JoinRoom',
  //     payload: { username: username.value }
  //   }));
  //   gameStatus.value = 'WAITING';
  // };
ws.onopen = () => {
  console.log('✅ 成功建立长连接');
  ws.send(JSON.stringify({
    type: 'JoinRoom',
    payload: { 
      username: username.value,
      color: '#10b981' // 🟢 补上 color 契约字段（先写死一个经典的绿色）
    }
  }));
  gameStatus.value = 'WAITING';
};

  ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    
    // 根据 lib.rs 的 ServerMessage 进行匹配处理
    switch (msg.type) {
      case 'RoomStatus':
        roomCode.value = msg.payload.room_code;
        players.value = msg.payload.players;
        tickRate.value = msg.payload.tick_rate_ms;
        break;
        
      case 'GameStart':
        gameStatus.value = 'PLAYING';
        // 变成 PLAYING 状态后，Canvas 节点才会被渲染，需要等下一个 DOM 刻度进行绘制初始化
        nextTick(() => {
          initCanvas();
        });
        break;
        
      case 'GameFrame':
        // 接收高频驱动的游戏帧画面，进行重绘
        drawGame(msg.payload);
        break;
        
      case 'GameOver':
        winner.value = msg.payload.winner_username || '无';
        roundScores.value = msg.payload.round_scores;
        showResult.value = true;
        gameStatus.value = 'WAITING';
        break;
        
      case 'Error':
        alert(`服务器异常: ${msg.payload.message}`);
        break;
    }
  };

  ws.onclose = () => {
    console.log('❌ 连线断开');
    gameStatus.value = 'IDLE';
  };
};

const sendReady = () => {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ type: 'Ready' })); //
  }
};

const leaveRoom = () => {
  if (ws && ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ type: 'LeaveRoom' })); //
    ws.close();
  }
  gameStatus.value = 'IDLE';
};

// --- 全局键盘事件拦截（解耦设计） ---
const handleKeyDown = (event) => {
  const action = KEY_MAP[event.key];
  if (!action) return;

  // 1. 如果处于战场状态，高频触发方向变换
  if (gameStatus.value === 'PLAYING') {
    if (['Up', 'Down', 'Left', 'Right'].includes(action)) {
      ws.send(JSON.stringify({
        type: 'ChangeDirection',
        payload: { direction: action } //
      }));
    }
    if (action === 'LEAVE') {
      leaveRoom();
    }
  } 
  // 2. 如果处于大厅准备状态，支持快捷键
  else if (gameStatus.value === 'WAITING') {
    if (action === 'READY') sendReady();
    if (action === 'LEAVE') leaveRoom();
  }
};

// --- Canvas 像素风渲染引擎 ---
let ctx = null;
const GRID_SIZE = 20; // 每个格子 20x20 像素

const initCanvas = () => {
  if (gameCanvas.value) {
    ctx = gameCanvas.value.getContext('2d');
  }
};

const drawGame = (gameState) => {
  if (!ctx || !gameCanvas.value) return;

  // 清空画布
  ctx.fillStyle = '#0f172a';
  ctx.fillRect(0, 0, gameCanvas.value.width, gameCanvas.value.height);

  // 1. 绘制食物 (亮红色像素方块)
  ctx.fillStyle = '#ef4444';
  ctx.fillRect(gameState.food.x * GRID_SIZE, gameState.food.y * GRID_SIZE, GRID_SIZE - 2, GRID_SIZE - 2);

  // 2. 绘制所有在线玩家的蛇
  gameState.snakes.forEach((snake) => {
    if (!snake.is_alive) return; // 挂了就不画了

    // 区分自己和敌人
    ctx.fillStyle = snake.username === username.value ? '#10b981' : '#3b82f6';

    snake.body.forEach((pos, index) => {
      // 蛇头稍微做点深浅区分，方便辨别方向
      if (index === 0) {
        ctx.fillStyle = snake.username === username.value ? '#34d399' : '#60a5fa';
      } else {
        ctx.fillStyle = snake.username === username.value ? '#10b981' : '#3b82f6';
      }
      ctx.fillRect(pos.x * GRID_SIZE, pos.y * GRID_SIZE, GRID_SIZE - 2, GRID_SIZE - 2);
    });
  });
};

// --- 生命周期钩子 ---
onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  if (ws) ws.close();
});
</script>