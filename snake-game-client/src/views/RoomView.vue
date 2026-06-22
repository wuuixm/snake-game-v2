<template>
    <div class="w-full max-w-5xl mx-auto p-6 space-y-5">
        <!-- 顶栏：房间信息 + 速度调节 -->
        <RoomInfoBar
            :room-code="state.roomCode"
            :mode="state.roomMode"
            :tick-rate-ms="state.tickRateMs"
            :is-host="isHost()"
            @refresh="refreshRoom"
            @set-speed="(val) => setSpeed(val)"
        />

        <!-- 主内容区：锦标赛（如有） + 成员/操作 -->
        <div class="grid grid-cols-1 lg:grid-cols-5 gap-5">
            <!-- 左侧区域（3/5）：锦标赛树或装饰 -->
            <div class="lg:col-span-3">
                <TournamentBracket
                    v-if="state.roomMode === 'Tournament'"
                    :tournament="state.tournament"
                />
                <!-- 经典模式的装饰占位 -->
                <div
                    v-else
                    class="fun-card h-full flex flex-col items-center justify-center text-center min-h-[220px]"
                >
                    <span class="text-5xl mb-3">🎮</span>
                    <h3 class="text-lg font-black text-fun-mint">经典模式</h3>
                    <p class="text-sm text-fun-gray mt-1">
                        准备好后点击「准备就绪」开始游戏
                    </p>
                    <div class="flex gap-2 mt-3 text-xs text-fun-gray">
                        <span
                            class="fun-badge bg-fun-mint/10 text-fun-mint border-fun-mint/30"
                            >🎯 自由练习</span
                        >
                        <span
                            class="fun-badge bg-fun-sky/10 text-fun-sky border-fun-sky/30"
                            >👥 最多 4 人</span
                        >
                    </div>
                </div>
            </div>

            <!-- 右侧（2/5）：成员列表 + 操作按钮 -->
            <div class="lg:col-span-2 space-y-5">
                <PlayerListCard
                    :players="state.players"
                    :current-user="state.username"
                    :host-username="state.hostUsername"
                    :show-waiting="tournamentWaiting()"
                />
                <RoomActionButtons
                    :disabled="tournamentWaiting()"
                    @ready="ready"
                    @leave="leaveRoom"
                />
            </div>
        </div>
    </div>
</template>

<script setup>
import { onMounted, onUnmounted } from "vue";
import { useGame } from "../composables/useGame";
import RoomInfoBar from "../components/room/RoomInfoBar.vue";
import TournamentBracket from "../components/room/TournamentBracket.vue";
import PlayerListCard from "../components/room/PlayerListCard.vue";
import RoomActionButtons from "../components/room/RoomActionButtons.vue";

const {
    state,
    ready,
    leaveRoom,
    setSpeed,
    isHost,
    tournamentWaiting,
    refreshRoom,
} = useGame();

const handleRoomKeys = (e) => {
    if (e.code === "Space") {
        e.preventDefault();
        ready();
    }
    if (e.code === "Escape") {
        e.preventDefault();
        leaveRoom();
    }
};

onMounted(() => window.addEventListener("keydown", handleRoomKeys));
onUnmounted(() => window.removeEventListener("keydown", handleRoomKeys));
</script>
