<template>
    <div class="relative">
        <canvas
            ref="gameCanvas"
            :width="canvasWidth"
            :height="canvasHeight"
            class="rounded-2xl border-2 border-fun-border bg-fun-soft"
        ></canvas>
    </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from "vue";
import { useGame } from "../../composables/useGame";

// ===================== 导入 SVG 图片资源 =====================
import appleSvg from "../../assets/Apple.svg";
import cherrySvg from "../../assets/Cherry.svg";
import orangeSvg from "../../assets/Orange.svg";
import grapeSvg from "../../assets/Grape.svg";
import melonSvg from "../../assets/Melon.svg";
import snakeRemainsSvg from "../../assets/SnakeRemains.svg";

const FOOD_SVG_IMPORTS = {
    Apple: appleSvg,
    Cherry: cherrySvg,
    Orange: orangeSvg,
    Grape: grapeSvg,
    Melon: melonSvg,
    SnakeRemains: snakeRemainsSvg,
};

const { state, changeDirection } = useGame();

// ===================== 渲染常量 =====================
const GRID_SIZE = 20;
const canvasWidth = ref(600);
const canvasHeight = ref(600);
const gameCanvas = ref(null);
let ctx = null;

// ===================== 预加载食物图片 =====================
const foodImages = {};
const loadFoodImages = () => {
    for (const [type, svgUrl] of Object.entries(FOOD_SVG_IMPORTS)) {
        const img = new Image();
        img.src = svgUrl;
        foodImages[type] = img;
    }
};

// ===================== 键盘控制 =====================
const DIRECTION_CODE_MAP = {
    KeyW: "Up",
    ArrowUp: "Up",
    KeyS: "Down",
    ArrowDown: "Down",
    KeyA: "Left",
    ArrowLeft: "Left",
    KeyD: "Right",
    ArrowRight: "Right",
};

const handleKeyDown = (event) => {
    const dir = DIRECTION_CODE_MAP[event.code];
    if (dir) {
        event.preventDefault();
        event.stopPropagation();
        changeDirection(dir);
    }
};

// ===================== Canvas 渲染 =====================
const drawGame = (frame) => {
    if (!ctx || !gameCanvas.value) return;

    // 清屏 - 浅色背景
    ctx.fillStyle = "#FFF8F0";
    ctx.fillRect(0, 0, gameCanvas.value.width, gameCanvas.value.height);

    // 网格线（淡色装饰）
    ctx.strokeStyle = "#F0EBE5";
    ctx.lineWidth = 0.5;
    for (let x = 0; x <= 30; x++) {
        ctx.beginPath();
        ctx.moveTo(x * GRID_SIZE, 0);
        ctx.lineTo(x * GRID_SIZE, 600);
        ctx.stroke();
    }
    for (let y = 0; y <= 30; y++) {
        ctx.beginPath();
        ctx.moveTo(0, y * GRID_SIZE);
        ctx.lineTo(600, y * GRID_SIZE);
        ctx.stroke();
    }

    // ==================== 食物（SVG 图片） ====================
    const drawFood = (pos, type) => {
        const img = foodImages[type || "Apple"];
        if (img && img.complete) {
            ctx.drawImage(
                img,
                pos.x * GRID_SIZE,
                pos.y * GRID_SIZE,
                GRID_SIZE,
                GRID_SIZE,
            );
        } else {
            // 后备：彩色圆点
            const colors = {
                Apple: "#FF6B6B",
                Cherry: "#FF6B9D",
                Orange: "#FF9F43",
                Grape: "#A29BFE",
                Melon: "#6BCB77",
                SnakeRemains: "#B2BEC3",
            };
            ctx.fillStyle = colors[type] || "#FF6B6B";
            ctx.beginPath();
            ctx.arc(
                pos.x * GRID_SIZE + 10,
                pos.y * GRID_SIZE + 10,
                6,
                0,
                2 * Math.PI,
            );
            ctx.fill();
        }
    };

    if (frame.foods) {
        frame.foods.forEach((food) => {
            drawFood(food.position, food.food_type || "Apple");
        });
    }
    if (frame.food && !frame.foods) {
        drawFood(frame.food, "Apple");
    }

    // ==================== 蛇群 ====================
    if (frame.snakes) {
        frame.snakes.forEach((snake) => {
            if (!snake.is_alive) return;
            const baseColor = snake.color || "#6BCB77";

            snake.body.forEach((pos, index) => {
                const x = pos.x * GRID_SIZE;
                const y = pos.y * GRID_SIZE;
                const totalLen = snake.body.length;

                if (index === 0) {
                    // === 蛇头：比身体大一圈 ===
                    const headSize = GRID_SIZE + 2;
                    const offset = -1; // 向左上偏移 1px 让头部突出

                    // 头部底色（白色外圈）
                    ctx.fillStyle = "#FFFFFF";
                    ctx.beginPath();
                    ctx.roundRect(
                        x + offset,
                        y + offset,
                        headSize,
                        headSize,
                        6,
                    );
                    ctx.fill();

                    // 头部主色
                    ctx.fillStyle = baseColor;
                    ctx.beginPath();
                    ctx.roundRect(
                        x + offset + 2,
                        y + offset + 2,
                        headSize - 4,
                        headSize - 4,
                        5,
                    );
                    ctx.fill();

                    // 眼睛 — 白色底 + 黑色瞳
                    const eyeY = y + 7;
                    ctx.fillStyle = "#FFFFFF";
                    ctx.beginPath();
                    ctx.arc(x + 5, eyeY, 3.5, 0, 2 * Math.PI);
                    ctx.arc(x + GRID_SIZE - 5, eyeY, 3.5, 0, 2 * Math.PI);
                    ctx.fill();

                    ctx.fillStyle = "#2D3436";
                    ctx.beginPath();
                    ctx.arc(x + 5, eyeY + 0.5, 2, 0, 2 * Math.PI);
                    ctx.arc(x + GRID_SIZE - 5, eyeY + 0.5, 2, 0, 2 * Math.PI);
                    ctx.fill();

                    // 眼睛高光
                    ctx.fillStyle = "#FFFFFF";
                    ctx.beginPath();
                    ctx.arc(x + 4, eyeY - 0.5, 0.8, 0, 2 * Math.PI);
                    ctx.arc(x + GRID_SIZE - 6, eyeY - 0.5, 0.8, 0, 2 * Math.PI);
                    ctx.fill();
                } else {
                    // === 蛇身：干净圆角方块 + 简单高光 ===
                    const segSize = GRID_SIZE - 2;
                    const segX = x + 1;
                    const segY = y + 1;

                    // 主体色
                    ctx.fillStyle = baseColor;
                    ctx.beginPath();
                    ctx.roundRect(segX, segY, segSize, segSize, 4);
                    ctx.fill();

                    // 高光点（左上角提亮，让蛇身有立体感）
                    ctx.fillStyle = "rgba(255,255,255,0.25)";
                    ctx.beginPath();
                    ctx.arc(segX + 5, segY + 5, 3, 0, 2 * Math.PI);
                    ctx.fill();

                    // 尾部略微缩小（靠近尾巴越来越细）
                    if (index > totalLen * 0.7) {
                        const shrink = 1 - (index / totalLen) * 0.3;
                        const smallSize = segSize * shrink;
                        const smallOffset = (segSize - smallSize) / 2;
                        ctx.fillStyle = baseColor;
                        ctx.beginPath();
                        ctx.roundRect(
                            segX + smallOffset,
                            segY + smallOffset,
                            smallSize,
                            smallSize,
                            3,
                        );
                        ctx.fill();
                    }
                }
            });
        });
    }
};

// 帧数据 → 自动重绘
watch(
    () => state.frameData,
    (newFrame) => {
        if (newFrame) drawGame(newFrame);
    },
    { deep: true },
);

// ===================== 生命周期 =====================
onMounted(() => {
    if (gameCanvas.value) ctx = gameCanvas.value.getContext("2d");
    loadFoodImages();
    window.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
    window.removeEventListener("keydown", handleKeyDown);
});
</script>
