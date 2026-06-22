<template>
    <!-- 速度调节环 — 放大版 -->
    <div class="flex flex-col items-center gap-1.5 select-none">
        <span class="text-xs font-black text-fun-gray uppercase tracking-wider"
            >⚡ 速度</span
        >
        <div
            class="relative w-24 h-24"
            @mousedown="startDrag"
            @touchstart.prevent="startDragTouch"
        >
            <svg
                viewBox="0 0 36 36"
                class="w-full h-full -rotate-90 drop-shadow-sm"
            >
                <!-- 背景环 -->
                <circle
                    cx="18"
                    cy="18"
                    r="15.5"
                    fill="none"
                    stroke="#F0EBE5"
                    stroke-width="3.5"
                />
                <!-- 进度环 -->
                <circle
                    cx="18"
                    cy="18"
                    r="15.5"
                    fill="none"
                    :stroke="progressColor"
                    stroke-width="3.5"
                    stroke-linecap="round"
                    :stroke-dasharray="circumference"
                    :stroke-dashoffset="dashOffset"
                    class="transition-all duration-150"
                />
                <!-- 中心点装饰 -->
                <circle cx="18" cy="18" r="2" fill="var(--color-fun-border)" />
            </svg>
            <!-- 中心数值 -->
            <div class="absolute inset-0 flex items-center justify-center">
                <span class="text-xs font-black text-fun-text">{{
                    displayValue
                }}</span>
            </div>
        </div>
        <!-- 文字标签 -->
        <div
            class="flex justify-between w-full text-[10px] text-fun-gray font-bold px-0.5"
        >
            <span>🐇 快</span>
            <span>🐢 慢</span>
        </div>
    </div>
</template>

<script setup>
import { computed } from "vue";

const props = defineProps({
    modelValue: { type: Number, default: 200 },
    min: { type: Number, default: 50 },
    max: { type: Number, default: 500 },
});
const emit = defineEmits(["update:modelValue"]);

const circumference = 2 * Math.PI * 15.5;

const fraction = computed(() => {
    return (props.modelValue - props.min) / (props.max - props.min);
});

const dashOffset = computed(() => {
    return circumference * (1 - fraction.value);
});

const displayValue = computed(() => `${props.modelValue}ms`);

const progressColor = computed(() => {
    const f = fraction.value;
    if (f < 0.33) return "#6BCB77";
    if (f < 0.66) return "#FFD93D";
    return "#FF6B6B";
});

const getAngle = (clientX, clientY, rect) => {
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height / 2;
    const dx = clientX - cx;
    const dy = clientY - cy;
    let angle = Math.atan2(dx, -dy);
    if (angle < 0) angle += 2 * Math.PI;
    return angle;
};

const setValueFromAngle = (angle) => {
    const f = Math.min(1, Math.max(0, angle / (2 * Math.PI)));
    const val = Math.round(props.min + f * (props.max - props.min));
    const stepped = Math.round(val / 25) * 25;
    emit(
        "update:modelValue",
        Math.min(props.max, Math.max(props.min, stepped)),
    );
};

const startDrag = (e) => {
    const el = e.currentTarget;
    const rect = el.getBoundingClientRect();
    setValueFromAngle(getAngle(e.clientX, e.clientY, rect));

    const onMove = (ev) => {
        setValueFromAngle(getAngle(ev.clientX, ev.clientY, rect));
    };
    const onUp = () => {
        window.removeEventListener("mousemove", onMove);
        window.removeEventListener("mouseup", onUp);
    };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
};

const startDragTouch = (e) => {
    const el = e.currentTarget;
    const rect = el.getBoundingClientRect();
    const touch = e.touches[0];
    setValueFromAngle(getAngle(touch.clientX, touch.clientY, rect));

    const onMove = (ev) => {
        ev.preventDefault();
        const t = ev.touches[0];
        setValueFromAngle(getAngle(t.clientX, t.clientY, rect));
    };
    const onUp = () => {
        window.removeEventListener("touchmove", onMove);
        window.removeEventListener("touchend", onUp);
    };
    window.addEventListener("touchmove", onMove, { passive: false });
    window.addEventListener("touchend", onUp);
};
</script>
