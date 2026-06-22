<template>
  <Teleport to="body">
    <div v-if="show" class="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center p-4 z-50" @click.self="$emit('close')">
      <div class="fun-card max-w-md w-full animate-bounce-in text-center space-y-4 border-fun-yellow border-2">
        <div class="text-5xl">👑</div>
        <h3 class="text-2xl font-black text-fun-orange tracking-wide">本局战况结算</h3>
        <p class="text-sm text-fun-gray">
          获胜者: <span class="text-fun-text font-black underline decoration-fun-yellow underline-offset-4">{{ winner }}</span>
        </p>

        <div class="bg-fun-soft rounded-2xl p-4 text-left border-2 border-fun-border max-h-48 overflow-y-auto">
          <div class="text-xs font-black text-fun-gray mb-2 uppercase tracking-wider border-b-2 border-fun-border pb-1 flex justify-between">
            <span>玩家名</span><span>本局得分</span>
          </div>
          <div v-for="score in scores" :key="score[0]" class="flex justify-between py-1.5 text-sm">
            <span :class="{'text-fun-orange font-black': score[0] === winner}" class="text-fun-text">{{ score[0] }}</span>
            <span class="font-bold text-fun-coral">+{{ score[1] }} 分</span>
          </div>
        </div>

        <button
          @click="$emit('backToRoom')"
          class="w-full py-3.5 fun-btn-yellow text-base flex items-center justify-center gap-2"
        >
          确认战绩并返回房间
        </button>
        <button
          @click="$emit('backToLobby')"
          class="w-full py-2.5 fun-btn-secondary flex items-center justify-center gap-2"
        >
          返回大厅
        </button>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
defineProps({
  show: Boolean,
  winner: String,
  scores: { type: Array, default: () => [] },
});
defineEmits(['close', 'backToRoom', 'backToLobby']);
</script>
