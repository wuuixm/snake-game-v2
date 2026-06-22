<template>
  <header class="relative z-10 bg-white/80 backdrop-blur-md border-b-2 border-fun-border px-6 py-3 flex justify-between items-center">
    <!-- 左侧 Logo -->
    <div class="flex items-center gap-3">
      <h1 class="text-lg font-black tracking-tight text-fun-text">
        <span class="text-fun-coral">Snake</span>
        <span class="text-fun-sky">Arena</span>
      </h1>
      <span class="hidden sm:inline text-xs font-bold text-fun-gray bg-fun-soft px-2 py-0.5 rounded-full">联机贪吃蛇</span>
    </div>

    <!-- 右侧用户信息 -->
    <div class="flex items-center gap-3">
      <template v-if="isLoggedIn">
        <div class="flex items-center gap-2 bg-fun-mint/10 border-2 border-fun-mint/30 px-3 py-1.5 rounded-full">
          <span class="text-sm">🎮</span>
          <span class="text-sm font-bold text-fun-mint">{{ username }}</span>
          <span
            class="w-2.5 h-2.5 rounded-full bg-fun-mint animate-pulse"
            title="已连接"
          ></span>
        </div>
        <button
          @click="$emit('logout')"
          class="fun-btn-secondary text-xs px-3 py-1.5 flex items-center gap-1"
          title="注销"
        >
          <span>🚪</span>
          <span class="hidden sm:inline">注销</span>
        </button>
      </template>
      <template v-else>
        <div class="flex items-center gap-2 bg-fun-soft border-2 border-fun-border px-3 py-1.5 rounded-full">
          <span class="text-xs font-bold text-fun-gray">未登录</span>
        </div>
      </template>
    </div>
  </header>
</template>

<script setup>
defineProps({
  isLoggedIn: Boolean,
  username: String,
});
defineEmits(['logout']);
</script>
