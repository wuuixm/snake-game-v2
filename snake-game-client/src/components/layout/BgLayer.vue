<template>
    <!-- 全屏背景层 + 大量装饰气泡（数量 ×1.5，大小 ×0.8） -->
    <div class="fixed inset-0 -z-10 overflow-hidden pointer-events-none">
        <img
            :src="bgImage"
            alt="background"
            class="w-full h-full object-cover opacity-50"
        />

        <!-- ====== 大号气泡 × 9 ====== -->
        <div
            class="absolute top-8 left-[5%] w-[88px] h-[88px] rounded-full bg-fun-yellow/50 animate-float"
        ></div>
        <div
            class="absolute top-28 right-[8%] w-[76px] h-[76px] rounded-full bg-fun-pink/45 animate-float"
            style="animation-delay: 0.6s; animation-duration: 4.5s"
        ></div>
        <div
            class="absolute bottom-20 left-[12%] w-[116px] h-[116px] rounded-full bg-fun-sky/40 animate-float"
            style="animation-delay: 1.2s"
        ></div>
        <div
            class="absolute bottom-36 right-[15%] w-[70px] h-[70px] rounded-full bg-fun-mint/50 animate-float"
            style="animation-delay: 0.3s; animation-duration: 3.5s"
        ></div>
        <div
            class="absolute top-[45%] left-[2%] w-[56px] h-[56px] rounded-full bg-fun-lavender/45 animate-float"
            style="animation-delay: 1s; animation-duration: 5s"
        ></div>
        <div
            class="absolute top-[25%] right-[3%] w-[84px] h-[84px] rounded-full bg-fun-orange/45 animate-float"
            style="animation-delay: 1.6s"
        ></div>
        <div
            class="absolute top-[60%] left-[18%] w-[64px] h-[64px] rounded-full bg-fun-coral/40 animate-float"
            style="animation-delay: 0.9s; animation-duration: 4s"
        ></div>
        <div
            class="absolute bottom-[10%] right-[5%] w-[78px] h-[78px] rounded-full bg-fun-teal/40 animate-float"
            style="animation-delay: 1.4s; animation-duration: 4.2s"
        ></div>
        <div
            class="absolute top-[12%] left-[40%] w-[48px] h-[48px] rounded-full bg-fun-pink/35 animate-float"
            style="animation-delay: 0.5s; animation-duration: 3.8s"
        ></div>

        <!-- ====== 中号气泡 × 15 ====== -->
        <div
            class="absolute top-[15%] left-[30%] w-8 h-8 rounded-full bg-fun-coral/40 animate-float"
            style="animation-delay: 0.4s; animation-duration: 3s"
        ></div>
        <div
            class="absolute bottom-[20%] right-[30%] w-[38px] h-[38px] rounded-full bg-fun-teal/40 animate-float"
            style="animation-delay: 1.3s; animation-duration: 3.8s"
        ></div>
        <div
            class="absolute top-[60%] left-[50%] w-[26px] h-[26px] rounded-full bg-fun-pink/35 animate-float"
            style="animation-delay: 2s; animation-duration: 3.2s"
        ></div>
        <div
            class="absolute top-[75%] left-[85%] w-[36px] h-[36px] rounded-full bg-fun-yellow/45 animate-float"
            style="animation-delay: 0.8s; animation-duration: 4.2s"
        ></div>
        <div
            class="absolute top-[8%] left-[55%] w-[28px] h-[28px] rounded-full bg-fun-mint/35 animate-float"
            style="animation-delay: 1.6s; animation-duration: 3.6s"
        ></div>
        <div
            class="absolute bottom-[35%] left-[42%] w-[22px] h-[22px] rounded-full bg-fun-lavender/40 animate-float"
            style="animation-delay: 2.2s; animation-duration: 4.5s"
        ></div>
        <div
            class="absolute bottom-[8%] left-[3%] w-[44px] h-[44px] rounded-full bg-fun-sky/35 animate-float"
            style="animation-delay: 0.5s"
        ></div>
        <div
            class="absolute top-[3%] right-[40%] w-8 h-8 rounded-full bg-fun-orange/40 animate-float"
            style="animation-delay: 1s; animation-duration: 4s"
        ></div>
        <div
            class="absolute top-[35%] left-[75%] w-[20px] h-[20px] rounded-full bg-fun-coral/35 animate-float"
            style="animation-delay: 1.9s; animation-duration: 3.3s"
        ></div>
        <div
            class="absolute bottom-[50%] right-[5%] w-[26px] h-[26px] rounded-full bg-fun-teal/35 animate-float"
            style="animation-delay: 0.9s; animation-duration: 3.9s"
        ></div>
        <div
            class="absolute top-[42%] left-[65%] w-[30px] h-[30px] rounded-full bg-fun-orange/35 animate-float"
            style="animation-delay: 1.1s; animation-duration: 3.5s"
        ></div>
        <div
            class="absolute bottom-[65%] right-[20%] w-[24px] h-[24px] rounded-full bg-fun-lavender/35 animate-float"
            style="animation-delay: 0.7s; animation-duration: 4.1s"
        ></div>
        <div
            class="absolute top-[20%] left-[80%] w-[32px] h-[32px] rounded-full bg-fun-mint/35 animate-float"
            style="animation-delay: 1.5s; animation-duration: 3.4s"
        ></div>
        <div
            class="absolute bottom-[5%] left-[55%] w-[18px] h-[18px] rounded-full bg-fun-coral/35 animate-float"
            style="animation-delay: 2.1s; animation-duration: 3.7s"
        ></div>
        <div
            class="absolute top-[55%] right-[35%] w-[28px] h-[28px] rounded-full bg-fun-sky/35 animate-float"
            style="animation-delay: 0.2s; animation-duration: 4.3s"
        ></div>
    </div>
</template>

<script setup>
import bgImage from "../../assets/bg.png";
</script>
