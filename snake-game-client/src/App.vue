<template>
    <div class="min-h-screen relative">
        <!-- 背景层 -->
        <BgLayer />

        <!-- 背景和内容之间的半透明分隔层 -->
        <div class="fixed inset-0 z-0 bg-white/50 pointer-events-none"></div>

        <!-- 顶栏 -->
        <GameHeader
            :is-logged-in="state.isLoggedIn"
            :username="state.username"
            @logout="logout"
        />

        <!-- 主体 -->
        <main
            class="relative z-10 py-6 px-4 flex items-center justify-center min-h-[85vh]"
        >
            <!-- 未登录：鉴权 -->
            <AuthView
                v-if="!state.isLoggedIn"
                @login="handleLogin"
                @register="handleRegister"
            />

            <!-- 已登录：按视图路由 -->
            <LobbyView v-else-if="state.currentView === 'LOBBY'" />
            <RoomView v-else-if="state.currentView === 'ROOM'" />
            <GameView v-else-if="state.currentView === 'PLAYING'" />
        </main>

        <!-- 全局结算弹窗（覆盖所有视图） -->
        <GameOverModal
            :show="state.matchResult.show"
            :winner="state.matchResult.winner"
            :scores="state.matchResult.scores"
            @close="closeGameOver"
            @back-to-room="closeGameOver"
            @back-to-lobby="backToLobbyFromGameOver"
        />
    </div>
</template>

<script setup>
import { useGame } from "./composables/useGame";
import BgLayer from "./components/layout/BgLayer.vue";
import GameHeader from "./components/layout/GameHeader.vue";
import AuthView from "./views/AuthView.vue";
import LobbyView from "./views/LobbyView.vue";
import RoomView from "./views/RoomView.vue";
import GameView from "./views/GameView.vue";
import GameOverModal from "./components/game/GameOverModal.vue";

const {
    state,
    login,
    register,
    logout,
    closeGameOver,
    backToLobbyFromGameOver,
} = useGame();

const handleLogin = async (username, password) => {
    const result = await login(username, password);
    if (!result.success) alert(result.message);
};

const handleRegister = async (username, password) => {
    const result = await register(username, password);
    alert(result.message);
};
</script>
