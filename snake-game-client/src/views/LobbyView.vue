<template>
    <div
        class="w-full max-w-5xl mx-auto grid grid-cols-1 lg:grid-cols-3 gap-4 p-6"
    >
        <!-- 左侧：玩家配置 + 房间列表 + 创建房间 -->
        <div class="lg:col-span-2 space-y-4">
            <PlayerConfigCard
                :username="state.username"
                :user-color="state.userColor"
                @update:user-color="state.userColor = $event"
            />

            <RoomListCard
                :rooms="state.roomList"
                @join="(code, mode) => joinRoom(code, mode)"
                @refresh="listRooms"
                @reset="handleReset"
            />

            <CreateRoomCard
                @create="({ code, mode }) => handleCreate(code, mode)"
            />
        </div>

        <!-- 右侧：全局积分榜 -->
        <LeaderboardCard
            :data="leaderboardData"
            :loading="loading"
            :current-user="state.username"
            @fetch="fetchLeaderboard"
        />
    </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { useGame } from "../composables/useGame";
import { httpUrl } from "../network/gameStream";
import PlayerConfigCard from "../components/lobby/PlayerConfigCard.vue";
import RoomListCard from "../components/lobby/RoomListCard.vue";
import CreateRoomCard from "../components/lobby/CreateRoomCard.vue";
import LeaderboardCard from "../components/lobby/LeaderboardCard.vue";

const { state, createRoom, listRooms, joinRoom, resetClient } = useGame();

const leaderboardData = ref([]);
const loading = ref(false);

const handleCreate = async (code, mode) => {
    const gameMode = mode === "TOURNAMENT" ? "Tournament" : "Classic";
    await createRoom(code, gameMode);
};

const handleReset = () => {
    resetClient();
    listRooms();
};

const fetchLeaderboard = async () => {
    loading.value = true;
    try {
        const res = await fetch(httpUrl("/api/leaderboard"));
        const json = await res.json();
        if (json.success) leaderboardData.value = json.data || [];
    } catch (err) {
        console.error("获取积分榜失败:", err);
    } finally {
        loading.value = false;
    }
};

onMounted(() => {
    fetchLeaderboard();
    listRooms();
});
</script>
