<template>
    <div class="flex flex-col items-center justify-center h-full select-none">
        <div
            class="relative w-full h-full bg-gradient-to-b from-fun-blue/15 via-fun-lavender/10 to-fun-pink/10 rounded-3xl border-2 border-fun-border/50 shadow-lg flex flex-col items-center pt-4 pb-3 px-3"
        >
            <!-- 顶部游戏信息 -->
            <div class="w-full mb-2 px-1 space-y-1.5 flex-shrink-0">
                <div
                    class="flex items-center justify-between bg-white/60 rounded-xl px-2.5 py-1.5 border border-fun-border/40"
                >
                    <span class="text-[10px] text-fun-gray font-bold"
                        >🏠 房间</span
                    >
                    <span class="text-xs font-black text-fun-sky font-mono">{{
                        roomCode
                    }}</span>
                </div>
                <div
                    class="flex items-center justify-between bg-white/60 rounded-xl px-2.5 py-1.5 border border-fun-border/40"
                >
                    <span class="text-[10px] text-fun-gray font-bold"
                        >⚡ 速度</span
                    >
                    <span class="text-xs font-black text-fun-coral"
                        >{{ tickRateMs }}ms</span
                    >
                </div>
                <div
                    v-if="isSpectating || hasForfeited"
                    class="flex items-center justify-center bg-fun-lavender/10 rounded-xl px-2.5 py-1 border border-fun-lavender/20"
                >
                    <span class="text-[10px] font-bold text-fun-lavender"
                        >👁️ 观战模式</span
                    >
                </div>
                <div
                    v-if="tournamentLabel"
                    class="flex items-center justify-center bg-fun-orange/10 rounded-xl px-2.5 py-1 border border-fun-orange/20"
                >
                    <span class="text-[10px] font-bold text-fun-orange">{{
                        tournamentLabel
                    }}</span>
                </div>
            </div>

            <div
                class="w-full border-t border-fun-border/40 mb-2 flex-shrink-0"
            ></div>

            <!-- SVG 摇杆 — 干净利落 -->
            <div
                class="flex-1 flex flex-col items-center justify-center w-full gap-3"
            >
                <!-- 摇杆 SVG -->
                <svg viewBox="0 0 120 120" class="w-32 h-32 drop-shadow-md">
                    <!-- 底座圆盘 -->
                    <circle
                        cx="60"
                        cy="68"
                        r="34"
                        fill="#F5F0EB"
                        stroke="#E8E0D8"
                        stroke-width="2.5"
                    />
                    <circle
                        cx="60"
                        cy="68"
                        r="26"
                        fill="#EBE5DD"
                        stroke="#DCD4CA"
                        stroke-width="1.5"
                    />
                    <!-- 底座高光 -->
                    <ellipse
                        cx="48"
                        cy="58"
                        rx="12"
                        ry="8"
                        fill="white"
                        opacity="0.3"
                    />
                    <!-- 摇杆杆 -->
                    <line
                        x1="60"
                        y1="68"
                        x2="52"
                        y2="24"
                        stroke="#FF6B6B"
                        stroke-width="5"
                        stroke-linecap="round"
                        opacity="0.7"
                    />
                    <line
                        x1="52"
                        y1="24"
                        x2="52"
                        y2="20"
                        stroke="#FF6B6B"
                        stroke-width="6"
                        stroke-linecap="round"
                    />
                    <!-- 摇杆球头 -->
                    <circle
                        cx="52"
                        cy="18"
                        r="13"
                        fill="url(#stickGrad)"
                        stroke="#FF6B6B"
                        stroke-width="2.5"
                    />
                    <ellipse
                        cx="48"
                        cy="13"
                        rx="5"
                        ry="4"
                        fill="white"
                        opacity="0.35"
                    />
                    <!-- 底座中心螺丝 -->
                    <circle
                        cx="60"
                        cy="68"
                        r="6"
                        fill="#DCD4CA"
                        stroke="#C8C0B6"
                        stroke-width="1.5"
                    />
                    <circle cx="60" cy="68" r="2" fill="#C8C0B6" />
                    <!-- 方向指示箭头 -->
                    <polygon points="60,38 57,46 63,46" fill="#B2BEC3" />
                    <polygon points="60,98 57,90 63,90" fill="#B2BEC3" />
                    <polygon points="30,68 38,65 38,71" fill="#B2BEC3" />
                    <polygon points="90,68 82,65 82,71" fill="#B2BEC3" />
                    <!-- 渐变定义 -->
                    <defs>
                        <radialGradient id="stickGrad" cx="40%" cy="35%">
                            <stop offset="0%" stop-color="#FF9E9E" />
                            <stop offset="100%" stop-color="#FF6B6B" />
                        </radialGradient>
                    </defs>
                </svg>

                <!-- 按键 SVG -->
                <svg viewBox="0 0 80 40" class="w-28 h-14">
                    <!-- A 键 -->
                    <circle
                        cx="22"
                        cy="20"
                        r="15"
                        fill="url(#aBtnGrad)"
                        stroke="#FF6B6B"
                        stroke-width="2.5"
                    />
                    <circle
                        cx="22"
                        cy="20"
                        r="15"
                        fill="none"
                        stroke="white"
                        stroke-width="1"
                        opacity="0.3"
                    />
                    <text
                        x="22"
                        y="25"
                        text-anchor="middle"
                        font-size="16"
                        font-weight="900"
                        fill="white"
                        font-family="sans-serif"
                    >
                        A
                    </text>
                    <!-- B 键 -->
                    <circle
                        cx="58"
                        cy="24"
                        r="13"
                        fill="url(#bBtnGrad)"
                        stroke="#4D96FF"
                        stroke-width="2.5"
                    />
                    <circle
                        cx="58"
                        cy="24"
                        r="13"
                        fill="none"
                        stroke="white"
                        stroke-width="1"
                        opacity="0.3"
                    />
                    <text
                        x="58"
                        y="29"
                        text-anchor="middle"
                        font-size="14"
                        font-weight="900"
                        fill="white"
                        font-family="sans-serif"
                    >
                        B
                    </text>
                    <!-- 脉冲光晕 A -->
                    <circle
                        cx="22"
                        cy="20"
                        r="18"
                        fill="none"
                        stroke="#FF6B6B"
                        stroke-width="1"
                        opacity="0.3"
                        class="animate-ping"
                    />
                    <!-- 脉冲光晕 B -->
                    <circle
                        cx="58"
                        cy="24"
                        r="16"
                        fill="none"
                        stroke="#4D96FF"
                        stroke-width="1"
                        opacity="0.3"
                        class="animate-ping"
                        style="animation-delay: 0.5s"
                    />
                    <defs>
                        <radialGradient id="aBtnGrad" cx="40%" cy="35%">
                            <stop offset="0%" stop-color="#FF9E9E" />
                            <stop offset="100%" stop-color="#FF6B6B" />
                        </radialGradient>
                        <radialGradient id="bBtnGrad" cx="40%" cy="35%">
                            <stop offset="0%" stop-color="#7EB8FF" />
                            <stop offset="100%" stop-color="#4D96FF" />
                        </radialGradient>
                    </defs>
                </svg>
            </div>

            <!-- 底部 -->
            <div
                class="flex-shrink-0 w-full flex flex-col items-center gap-1 mt-1"
            >
                <div
                    class="flex items-center gap-1 bg-fun-mint/10 rounded-full px-3 py-1 border border-fun-mint/20"
                >
                    <span
                        class="w-2 h-2 rounded-full bg-fun-mint animate-pulse"
                    ></span>
                    <span class="text-[9px] font-bold text-fun-mint"
                        >实时同步中</span
                    >
                </div>
                <span
                    class="text-[8px] text-fun-gray font-black uppercase tracking-widest"
                    >▶ PLAYER 1 ◀</span
                >
            </div>
        </div>
    </div>
</template>

<script setup>
defineProps({
    roomCode: String,
    tickRateMs: Number,
    tournamentLabel: { type: String, default: "" },
    isSpectating: { type: Boolean, default: false },
    hasForfeited: { type: Boolean, default: false },
});
</script>
