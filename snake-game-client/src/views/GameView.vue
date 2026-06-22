<template>
    <div class="w-full max-w-6xl mx-auto p-4 space-y-3">
        <!-- 三栏 Grid 布局：左右两栏固定宽高完全一致 -->
        <div
            class="grid grid-cols-[14rem_1fr_14rem] gap-5 justify-items-center"
        >
            <!-- 左侧：装饰摇杆（固定 w-56 = 14rem，高 450px） -->
            <div class="w-56 h-[450px]">
                <ArcadeDecor
                    :room-code="state.roomCode"
                    :tick-rate-ms="state.tickRateMs"
                    :tournament-label="tournamentStageLabel()"
                    :is-spectating="isSpectator()"
                    :has-forfeited="forfeited"
                />
            </div>

            <!-- 中间：游戏画布 -->
            <div class="flex-shrink-0">
                <div
                    class="fun-card !p-3 !border-2 !border-fun-mint/30 shadow-lg"
                >
                    <GameCanvas ref="canvasRef" />
                </div>
            </div>

            <!-- 右侧：实时分数（固定 w-56 = 14rem，高 450px，与左侧完全相同） -->
            <div class="w-56 h-[450px]">
                <ScorePanel
                    :snakes="allSnakes"
                    :current-user="state.username"
                />
            </div>
        </div>

        <!-- 底栏：操作按钮 -->
        <div class="flex justify-center">
            <GameBottomBar @forfeit="handleForfeit" />
        </div>
    </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useGame } from "../composables/useGame";
import GameCanvas from "../components/game/GameCanvas.vue";
import GameBottomBar from "../components/game/GameBottomBar.vue";
import ArcadeDecor from "../components/game/ArcadeDecor.vue";
import ScorePanel from "../components/game/ScorePanel.vue";

const { state, forfeit, tournamentStageLabel, isSpectator } = useGame();

const canvasRef = ref(null);
const forfeited = ref(false);

const handleForfeit = () => {
    if (!forfeited.value) {
        forfeit();
        forfeited.value = true;
    }
};

const handleGameKeys = (e) => {
    if (e.code === "Escape") {
        e.preventDefault();
        handleForfeit();
    }
};

onMounted(() => window.addEventListener("keydown", handleGameKeys));
onUnmounted(() => window.removeEventListener("keydown", handleGameKeys));

const allSnakes = computed(() => {
    if (!state.frameData?.snakes) return [];
    return state.frameData.snakes.map((s) => ({
        username: s.username,
        score: s.score || 0,
        is_alive: s.is_alive,
        color: s.color,
    }));
});
</script>
