<template>
    <div class="fun-card-accent-hover" style="border-left-color: #4d96ff">
        <label
            class="block text-xs font-black text-fun-sky uppercase tracking-wider mb-2 flex items-center gap-1.5"
        >
            服务器地址
        </label>
        <input
            :value="modelValue"
            @input="$emit('update:modelValue', $event.target.value)"
            @change="$emit('change', $event.target.value)"
            type="text"
            class="fun-input"
            placeholder="localhost:8080"
        />
    </div>
</template>

<script setup>
defineProps({
    modelValue: String,
});
defineEmits(["update:modelValue", "change"]);
</script>
