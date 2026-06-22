<template>
    <!-- 右侧分数面板 — 固定高度与左侧对称 -->
    <div
        class="fun-card h-full flex flex-col border-2 border-fun-sky/20 shadow-md"
    >
        <h3
            class="text-sm font-black text-fun-text mb-2.5 pb-2 border-b-2 border-fun-border flex items-center gap-2 flex-shrink-0"
        >
            实时积分
            <span
                class="text-[10px] font-normal text-fun-gray ml-auto fun-badge bg-fun-soft border-fun-border"
            >
                {{ aliveCount }}/{{ totalCount }} 存活
            </span>
        </h3>

        <div class="flex-1 space-y-2 overflow-y-auto min-h-0">
            <div
                v-for="snake in sortedSnakes"
                :key="snake.username"
                class="flex items-center justify-between p-2.5 rounded-xl border-2 transition-all duration-300"
                :class="
                    snake.is_alive
                        ? 'bg-fun-mint/5 border-fun-mint/20'
                        : 'bg-fun-soft border-fun-border opacity-60'
                "
            >
                <div class="flex items-center gap-2">
                    <span class="text-base">{{
                        snake.is_alive ? "🐍" : "💀"
                    }}</span>
                    <span
                        class="text-xs font-bold"
                        :class="
                            snake.is_alive
                                ? 'text-fun-text'
                                : 'text-fun-gray line-through'
                        "
                    >
                        {{
                            snake.username === currentUser
                                ? "⭐ 你"
                                : snake.username
                        }}
                    </span>
                </div>
                <span
                    class="text-sm font-black tabular-nums"
                    :class="snake.is_alive ? 'text-fun-coral' : 'text-fun-gray'"
                >
                    {{ snake.score || 0 }}
                </span>
            </div>

            <div
                v-if="snakes.length === 0"
                class="flex items-center justify-center h-[200px] text-fun-gray text-sm"
            >
                <div class="text-center space-y-2">
                    <span class="text-3xl">🌀</span>
                    <p>等待数据...</p>
                </div>
            </div>
        </div>

        <div class="mt-2 pt-2 border-t-2 border-fun-border flex-shrink-0">
            <div
                class="text-[10px] text-fun-gray font-bold text-center leading-relaxed"
            >
                🎮 W/A/S/D &nbsp;·&nbsp; ↑/↓/←/→
            </div>
        </div>
    </div>
</template>

<script setup>
import { computed } from "vue";

const props = defineProps({
    snakes: { type: Array, default: () => [] },
    currentUser: String,
});

const sortedSnakes = computed(() => {
    return [...props.snakes].sort((a, b) => (b.score || 0) - (a.score || 0));
});

const aliveCount = computed(
    () => props.snakes.filter((s) => s.is_alive).length,
);
const totalCount = computed(() => props.snakes.length);
</script>
