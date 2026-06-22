<template>
    <div class="fun-card-accent-hover !p-4" style="border-left-color: #ffd93d">
        <div class="flex justify-between items-center mb-3">
            <h3
                class="text-base font-black text-fun-orange flex items-center gap-2"
            >
                可加入的房间
            </h3>
            <div class="flex items-center gap-2">
                <button
                    @click="handleReset"
                    class="fun-btn-primary text-[10px] px-2 py-1 flex items-center gap-1"
                    title="一键重置所有状态"
                >
                    重置
                </button>
                <button
                    @click="$emit('refresh')"
                    class="fun-btn-secondary text-[10px] px-2 py-1"
                >
                    刷新
                </button>
            </div>
        </div>

        <div
            class="bg-fun-soft rounded-2xl border-2 border-fun-border overflow-y-auto min-h-[90px] max-h-[200px]"
        >
            <div
                v-if="rooms.length === 0"
                class="p-4 text-center text-fun-gray text-sm"
            >
                暂无可用房间，创建一个新房间吧！
            </div>
            <div v-else class="divide-y-2 divide-fun-border">
                <div
                    v-for="room in rooms"
                    :key="room.room_code"
                    class="flex items-center justify-between p-2.5 hover:bg-fun-yellow/10 transition cursor-pointer"
                    @click="$emit('join', room.room_code, room.mode)"
                >
                    <div class="flex items-center gap-2.5">
                        <span class="font-black text-fun-coral font-mono">{{
                            room.room_code
                        }}</span>
                        <span
                            :class="
                                room.mode === 'Tournament'
                                    ? 'bg-fun-orange/10 text-fun-orange border-fun-orange/30'
                                    : 'bg-fun-mint/10 text-fun-mint border-fun-mint/30'
                            "
                            class="text-[10px] px-2 py-0.5 rounded-full border-2 font-bold"
                        >
                            {{
                                room.mode === "Tournament"
                                    ? "🏆 锦标赛"
                                    : "🎮 经典"
                            }}
                        </span>
                    </div>
                    <div class="flex items-center gap-2 text-[10px]">
                        <span class="text-fun-gray"
                            >👑 {{ room.host_username }}</span
                        >
                        <span
                            class="font-bold"
                            :class="
                                room.player_count >= room.max_players
                                    ? 'text-fun-coral'
                                    : 'text-fun-mint'
                            "
                        >
                            {{ room.player_count }}/{{ room.max_players }}
                        </span>
                        <button class="fun-btn-green text-[10px] px-2 py-0.5">
                            加入
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
const emit = defineEmits(["join", "refresh", "reset"]);

defineProps({
    rooms: { type: Array, default: () => [] },
});

const handleReset = () => {
    if (
        confirm(
            "确定要重置所有状态吗？\n这将断开当前连接并清空所有房间/游戏状态。",
        )
    ) {
        emit("reset");
    }
};
</script>
