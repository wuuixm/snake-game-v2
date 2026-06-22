<template>
  <div class="fun-card flex justify-between items-center !rounded-b-none !border-b-0">
    <div class="flex items-center gap-3 text-sm">
      <span class="font-black text-fun-sky">房间: {{ roomCode }}</span>
      <span v-if="tournamentLabel" class="text-fun-orange font-black animate-pulse">
        {{ tournamentLabel }}
      </span>
      <span v-if="isSpectating || hasForfeited" class="text-xs bg-fun-lavender/10 text-fun-lavender px-2 py-0.5 rounded-full border-2 border-fun-lavender/30 font-bold">
        👁️ 观战模式
      </span>
      <span class="text-fun-border">|</span>
      <span class="text-fun-gray font-bold">⚡ {{ tickRateMs }}ms/帧</span>
    </div>
    <div class="flex items-center gap-2">
      <span class="w-2.5 h-2.5 rounded-full bg-fun-mint animate-pulse"></span>
      <span class="text-xs font-bold text-fun-mint">实时同步中</span>
    </div>
  </div>
</template>

<script setup>
defineProps({
  roomCode: String,
  tournamentLabel: String,
  isSpectating: Boolean,
  hasForfeited: Boolean,
  tickRateMs: Number,
});
</script>
