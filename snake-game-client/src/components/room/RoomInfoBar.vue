<template>
  <div class="fun-card flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
    <div>
      <span
        :class="mode === 'Tournament'
          ? 'bg-fun-orange/10 text-fun-orange border-fun-orange/30'
          : 'bg-fun-mint/10 text-fun-mint border-fun-mint/30'"
        class="text-xs px-2.5 py-1 rounded-full border-2 font-bold inline-block"
      >
        {{ mode === 'Tournament' ? '🏆 锦标赛' : '🎮 经典模式' }}
      </span>
      <div class="flex items-center gap-3 mt-1.5">
        <h2 class="text-xl font-black text-fun-text">
          房间 <span class="text-fun-sky font-mono">{{ roomCode }}</span>
        </h2>
        <button
          @click="$emit('refresh')"
          class="fun-btn-secondary text-xs px-2 py-1"
          title="刷新房间状态"
        >
         刷新 
        </button>
      </div>
    </div>

    <!-- 速度调节（圆环） -->
    <div v-if="isHost" class="flex items-center gap-2">
      <CircularSpeedControl
        :model-value="tickRateMs"
        @update:model-value="$emit('setSpeed', $event)"
      />
    </div>
    <div v-else class="bg-fun-soft rounded-2xl px-3 py-2 border-2 border-fun-border text-xs text-fun-gray font-bold">
      ⚡ 仅房主可调速度<br/>
      <span class="text-fun-text">当前: {{ tickRateMs }}ms/帧</span>
    </div>
  </div>
</template>

<script setup>
import CircularSpeedControl from './CircularSpeedControl.vue';

defineProps({
  roomCode: String,
  mode: String,
  tickRateMs: Number,
  isHost: Boolean,
});
defineEmits(['refresh', 'setSpeed']);
</script>
