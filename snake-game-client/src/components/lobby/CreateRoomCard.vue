<template>
    <div class="fun-card-accent-hover !p-4" style="border-left-color: #4d96ff">
        <h3
            class="text-base font-black text-fun-sky mb-3 flex items-center gap-2"
        >
            创建新房间
        </h3>

        <div class="space-y-3">
            <div>
                <label
                    class="block text-xs font-bold text-fun-gray uppercase tracking-wider mb-1"
                    >房间码（留空自动生成）</label
                >
                <input
                    v-model="localCode"
                    type="text"
                    maxlength="8"
                    class="fun-input uppercase"
                    placeholder="例如: ABCD"
                />
            </div>

            <div>
                <label
                    class="block text-xs font-bold text-fun-gray uppercase tracking-wider mb-1.5"
                    >选择赛制</label
                >
                <div class="grid grid-cols-2 gap-2">
                    <button
                        @click="localMode = 'CLASSIC'"
                        :class="
                            localMode === 'CLASSIC'
                                ? 'bg-fun-mint/10 border-fun-mint text-fun-mint'
                                : 'border-fun-border text-fun-gray hover:border-fun-gray'
                        "
                        class="py-2 border-2 rounded-2xl font-bold transition flex flex-col items-center gap-1"
                    >
                        <span class="text-lg">🎮</span>
                        <span class="text-xs">经典练习室</span>
                    </button>
                    <button
                        @click="localMode = 'TOURNAMENT'"
                        :class="
                            localMode === 'TOURNAMENT'
                                ? 'bg-fun-orange/10 border-fun-orange text-fun-orange'
                                : 'border-fun-border text-fun-gray hover:border-fun-gray'
                        "
                        class="py-2 border-2 rounded-2xl font-bold transition flex flex-col items-center gap-1"
                    >
                        <span class="text-lg">🏆</span>
                        <span class="text-xs">四人争霸赛</span>
                    </button>
                </div>
            </div>

            <button
                @click="
                    $emit('create', {
                        code: localCode.trim().toUpperCase() || null,
                        mode: localMode,
                    })
                "
                class="w-full py-3 fun-btn-sky text-sm flex items-center justify-center gap-2"
            >
                创建房间
            </button>
        </div>
    </div>
</template>

<script setup>
import { ref } from "vue";

const emit = defineEmits(["create"]);
const localCode = ref("");
const localMode = ref("CLASSIC");
</script>
