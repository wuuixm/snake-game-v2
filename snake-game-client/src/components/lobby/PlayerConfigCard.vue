<template>
    <div class="fun-card-accent-hover !p-4" style="border-left-color: #6bcb77">
        <h2
            class="text-base font-black text-fun-mint mb-3 flex items-center gap-2"
        >
            玩家配置
        </h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
                <label
                    class="block text-xs font-bold text-fun-gray uppercase tracking-wider mb-1.5"
                    >玩家昵称</label
                >
                <div
                    class="fun-badge !text-sm !px-4 !py-2.5 bg-fun-mint/10 text-fun-mint border-fun-mint/30 w-full justify-center"
                >
                    🎮 {{ username }}
                </div>
            </div>
            <div>
                <label
                    class="block text-xs font-bold text-fun-gray uppercase tracking-wider mb-1.5"
                    >自选蛇身颜色</label
                >
                <div
                    class="flex items-center gap-3 bg-fun-soft rounded-2xl p-2 border-2 border-fun-border"
                >
                    <input
                        :value="userColor"
                        @input="$emit('update:userColor', $event.target.value)"
                        type="color"
                        class="w-10 h-10 bg-transparent border-0 cursor-pointer rounded-xl"
                    />
                    <span class="text-sm font-bold text-fun-text">{{
                        userColor.toUpperCase()
                    }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup>
defineProps({
    username: String,
    userColor: String,
});
defineEmits(["update:userColor"]);
</script>
