<template>
  <div class="fun-card flex flex-col justify-between">
    <div>
      <h3 class="text-md font-black text-fun-text mb-3 pb-2 border-b-2 border-fun-border flex justify-between items-center">
        <span>👥 成员列表</span>
        <span class="text-xs bg-fun-soft px-2 py-0.5 rounded-full text-fun-gray font-bold">{{ players.length }}/4</span>
      </h3>

      <div class="space-y-2.5">
        <div
          v-for="p in players"
          :key="p.username"
          class="flex items-center justify-between p-3 bg-fun-soft rounded-2xl border-2 border-fun-border transition hover:border-fun-yellow/50"
        >
          <div class="flex items-center gap-2.5">
            <span class="font-bold text-fun-text text-sm">{{ p.username }}</span>
            <span
              v-if="p.username === currentUser"
              class="text-[10px] bg-fun-sky/10 text-fun-sky px-1.5 py-0.5 rounded-full border-2 border-fun-sky/30 font-bold"
            >你</span>
            <span
              v-if="p.username === hostUsername"
              class="text-[10px] bg-fun-yellow/20 text-fun-orange px-1.5 py-0.5 rounded-full border-2 border-fun-yellow/30 font-bold"
            >👑 房主</span>
          </div>
          <span
            :class="p.is_ready
              ? 'bg-fun-mint/10 text-fun-mint border-fun-mint/30'
              : 'bg-fun-pink/10 text-fun-pink border-fun-pink/30'"
            class="text-xs px-2.5 py-1 rounded-full font-bold border-2"
          >
            {{ p.is_ready ? '✅ READY' : '⏳ WAITING' }}
          </span>
        </div>
      </div>
    </div>

    <!-- 锦标赛等待提示 -->
    <div
      v-if="showWaiting"
      class="mt-4 text-xs bg-fun-orange/10 border-2 border-fun-orange/30 text-fun-orange px-3 py-2 rounded-2xl text-center font-bold"
    >
      ⏳ 等待中... {{ players.length }}/4 名玩家已就绪
    </div>
  </div>
</template>

<script setup>
defineProps({
  players: { type: Array, default: () => [] },
  currentUser: String,
  hostUsername: String,
  showWaiting: Boolean,
});
</script>
