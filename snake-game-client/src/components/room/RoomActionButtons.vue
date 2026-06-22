<template>
  <div class="space-y-3">
    <button
      @click="$emit('ready')"
      :disabled="disabled"
      :class="disabled
        ? 'bg-fun-gray/30 text-fun-gray cursor-not-allowed border-fun-border'
        : 'fun-btn-green'"
      class="w-full py-3.5 text-base flex items-center justify-center gap-2"
    >
      <span>🏁</span>
      {{ disabled ? '⏳ 等待凑齐 4 人...' : '准备就绪 (Space)' }}
    </button>

    <button
      @click="$emit('leave')"
      class="w-full py-2.5 fun-btn-secondary flex items-center justify-center gap-2"
    >
      <span>🚪</span> 退出房间 (Esc)
    </button>
  </div>
</template>

<script setup>
defineProps({
  disabled: Boolean,
});
defineEmits(['ready', 'leave']);
</script>
