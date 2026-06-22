<template>
  <div class="fun-card-accent" style="border-left-color: #FF9F43">
    <h3 class="text-md font-black text-fun-orange mb-4 flex items-center gap-2">
      <span>🏆</span> 锦标赛晋级图
    </h3>

    <div class="grid grid-cols-[1fr_auto_1fr] gap-3 items-center min-h-[180px] bg-fun-soft rounded-2xl p-4 border-2 border-fun-border">
      <!-- 左侧：半决赛 -->
      <div class="space-y-6">
        <div class="space-y-1.5">
          <div class="text-[10px] text-fun-gray font-black uppercase tracking-wider">半决赛 A 组</div>
          <div :class="['fun-card !p-2 !rounded-xl flex justify-between text-xs', highlightA ? 'border-fun-mint bg-fun-mint/5' : '']">
            <span :class="highlightA ? 'text-fun-mint font-black' : 'text-fun-text'">{{ bracketA1 }}</span>
            <span class="text-fun-gray font-bold">P1</span>
          </div>
          <div :class="['fun-card !p-2 !rounded-xl flex justify-between text-xs', highlightA ? 'border-fun-coral bg-fun-coral/5' : '']">
            <span :class="highlightA ? 'text-fun-coral font-black' : 'text-fun-text'">{{ bracketA2 }}</span>
            <span class="text-fun-gray font-bold">P2</span>
          </div>
        </div>
        <div class="space-y-1.5">
          <div class="text-[10px] text-fun-gray font-black uppercase tracking-wider">半决赛 B 组</div>
          <div :class="['fun-card !p-2 !rounded-xl flex justify-between text-xs', highlightB ? 'border-fun-sky bg-fun-sky/5' : '']">
            <span :class="highlightB ? 'text-fun-sky font-black' : 'text-fun-text'">{{ bracketB1 }}</span>
            <span class="text-fun-gray font-bold">P3</span>
          </div>
          <div :class="['fun-card !p-2 !rounded-xl flex justify-between text-xs', highlightB ? 'border-fun-lavender bg-fun-lavender/5' : '']">
            <span :class="highlightB ? 'text-fun-lavender font-black' : 'text-fun-text'">{{ bracketB2 }}</span>
            <span class="text-fun-gray font-bold">P4</span>
          </div>
        </div>
      </div>

      <!-- 中间：箭头 -->
      <div class="flex flex-col justify-around h-full text-fun-border text-xl select-none">
        <div>➔</div>
        <div>➔</div>
      </div>

      <!-- 右侧：总决赛 -->
      <div class="flex flex-col justify-center h-full space-y-1.5">
        <div class="text-[10px] text-fun-orange font-black uppercase tracking-wider text-center">👑 总决赛</div>
        <div :class="['fun-card !p-3 !rounded-2xl space-y-2', highlightFinal ? 'border-fun-orange bg-fun-orange/5' : 'border-fun-yellow/50']">
          <div :class="['px-2.5 py-2 rounded-xl text-xs text-center border-2 font-bold',
            isFinalistSet(0)
              ? 'bg-fun-orange/10 border-fun-orange/30 text-fun-orange'
              : 'bg-fun-soft border-fun-border text-fun-gray italic' ]">
            {{ finalistA }}
          </div>
          <div :class="['px-2.5 py-2 rounded-xl text-xs text-center border-2 font-bold',
            isFinalistSet(1)
              ? 'bg-fun-orange/10 border-fun-orange/30 text-fun-orange'
              : 'bg-fun-soft border-fun-border text-fun-gray italic' ]">
            {{ finalistB }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';

const props = defineProps({
  tournament: { type: Object, default: () => ({
    bracketA: ['', ''],
    bracketB: ['', ''],
    winnerA: null,
    winnerB: null,
    stage: '',
  })},
});

const bracketA1 = computed(() => props.tournament.bracketA[0] || '等待加入...');
const bracketA2 = computed(() => props.tournament.bracketA[1] || '等待加入...');
const bracketB1 = computed(() => props.tournament.bracketB[0] || '等待加入...');
const bracketB2 = computed(() => props.tournament.bracketB[1] || '等待加入...');
const finalistA = computed(() => props.tournament.winnerA || 'A组胜者 (待定)');
const finalistB = computed(() => props.tournament.winnerB || 'B组胜者 (待定)');

const highlightA = computed(() => props.tournament.stage === 'SemifinalA');
const highlightB = computed(() => props.tournament.stage === 'SemifinalB');
const highlightFinal = computed(() => props.tournament.stage === 'Final');

const isFinalistSet = (idx) => {
  const name = idx === 0 ? props.tournament.winnerA : props.tournament.winnerB;
  return name && !name.includes('待定');
};
</script>
