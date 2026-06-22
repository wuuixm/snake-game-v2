<template>
    <div
        class="fun-card-accent-hover"
        :style="{ borderLeftColor: mode === 'login' ? '#FF6B9D' : '#FFD93D' }"
    >
        <!-- 切换标签 -->
        <div class="flex mb-5 bg-fun-soft rounded-2xl p-1">
            <button
                @click="$emit('update:mode', 'login')"
                :class="
                    mode === 'login'
                        ? 'bg-white shadow-sm text-fun-pink font-black'
                        : 'text-fun-gray hover:text-fun-text'
                "
                class="flex-1 py-2.5 rounded-xl text-sm font-bold transition-all duration-200"
            >
                💖 登录
            </button>
            <button
                @click="$emit('update:mode', 'register')"
                :class="
                    mode === 'register'
                        ? 'bg-white shadow-sm text-fun-orange font-black'
                        : 'text-fun-gray hover:text-fun-text'
                "
                class="flex-1 py-2.5 rounded-xl text-sm font-bold transition-all duration-200"
            >
                ✨ 注册
            </button>
        </div>

        <!-- 表单 -->
        <form
            @submit.prevent="
                $emit('submit', {
                    username: formUsername,
                    password: formPassword,
                })
            "
            class="space-y-4"
        >
            <div>
                <label
                    class="block text-xs font-bold text-fun-gray uppercase tracking-wider mb-1.5"
                    >账号</label
                >
                <input
                    v-model="formUsername"
                    type="text"
                    required
                    class="fun-input"
                    placeholder="输入你的账户名称..."
                />
            </div>
            <div>
                <label
                    class="block text-xs font-bold text-fun-gray uppercase tracking-wider mb-1.5"
                    >密码</label
                >
                <input
                    v-model="formPassword"
                    type="password"
                    required
                    class="fun-input"
                    placeholder="••••••••"
                />
            </div>
            <button
                type="submit"
                :class="mode === 'login' ? 'fun-btn-primary' : 'fun-btn-yellow'"
                class="w-full py-3.5 text-base flex items-center justify-center gap-2"
            >
                {{ mode === "login" ? "登录游戏" : "注册账号" }}
            </button>
        </form>
    </div>
</template>

<script setup>
import { ref, watch } from "vue";

const props = defineProps({
    mode: { type: String, default: "login" },
});
const emit = defineEmits(["update:mode", "submit"]);

const formUsername = ref("");
const formPassword = ref("");

// 切换 mode 时清空表单
watch(
    () => props.mode,
    () => {
        formUsername.value = "";
        formPassword.value = "";
    },
);
</script>
