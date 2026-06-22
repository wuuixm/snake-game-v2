<template>
    <div
        class="fun-card-accent-hover !p-4 flex flex-col"
        style="border-left-color: #a29bfe"
    >
        <div class="flex justify-between items-center mb-3">
            <h3
                class="text-base font-black text-fun-lavender flex items-center gap-2"
            >
                全球积分榜
            </h3>
            <button
                @click="$emit('fetch')"
                class="fun-btn-secondary text-[10px] px-2 py-1"
            >
                刷新
            </button>
        </div>

        <div
            class="flex-1 bg-fun-soft rounded-2xl border-2 border-fun-border overflow-y-auto p-2 min-h-[250px]"
        >
            <div
                v-if="loading"
                class="h-full flex items-center justify-center text-fun-gray text-sm"
            >
                🌀 正在读取全服数据...
            </div>
            <div
                v-else-if="data.length === 0"
                class="h-full flex items-center justify-center text-fun-gray text-sm"
            >
                🌟 暂无英雄上榜，快去开一局！
            </div>
            <div v-else class="space-y-1.5">
                <div
                    v-for="entry in data"
                    :key="entry.rank"
                    :class="
                        entry.username === currentUser
                            ? 'bg-fun-mint/10 border-fun-mint/30'
                            : 'border-transparent'
                    "
                    class="flex items-center justify-between p-2.5 border-2 rounded-xl text-sm hover:bg-fun-yellow/10 transition"
                >
                    <div class="flex items-center gap-3">
                        <span
                            class="w-6 text-center font-black"
                            :class="medalClass(entry.rank)"
                        >
                            {{
                                entry.rank <= 3
                                    ? medals[entry.rank - 1]
                                    : entry.rank
                            }}
                        </span>
                        <span class="font-bold text-fun-text">{{
                            entry.username
                        }}</span>
                    </div>
                    <div class="flex items-center gap-3 text-xs">
                        <span class="text-fun-gray"
                            >场次: {{ entry.total_games }}</span
                        >
                        <span class="text-fun-orange font-black"
                            >⭐ {{ entry.max_score }}</span
                        >
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
const medals = ["🥇", "🥈", "🥉"];
const medalClass = (rank) => {
    if (rank === 1) return "text-fun-yellow";
    if (rank === 2) return "text-fun-gray";
    if (rank === 3) return "text-fun-orange";
    return "text-fun-gray";
};

defineProps({
    data: { type: Array, default: () => [] },
    loading: Boolean,
    currentUser: String,
});
defineEmits(["fetch"]);
</script>
