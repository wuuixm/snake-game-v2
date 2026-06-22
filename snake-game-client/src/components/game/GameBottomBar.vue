<template>
    <div class="flex items-center gap-3">
        <button
            @click="$emit('forfeit')"
            class="fun-btn-primary text-sm px-4 py-2 flex items-center gap-2"
        >
            <span>🏳️</span> 投降观战 (Esc)
        </button>
        <div
            class="text-xs text-fun-gray px-3 py-1.5 bg-fun-soft rounded-full border-2 border-fun-border"
        >
            🎮 <span class="text-fun-text font-bold">W/A/S/D</span> 或
            <span class="text-fun-text font-bold">↑/↓/←/→</span>
        </div>
    </div>
</template>

<script setup>
defineEmits(["forfeit"]);
</script>
