<template>
  <div class="w-full max-w-md mx-auto space-y-5">
    <!-- 服务器地址 -->
    <ServerAddressInput
      :model-value="serverAddr"
      @update:model-value="serverAddr = $event"
      @change="onAddrChange"
    />

    <!-- 鉴权表单 -->
    <AuthForm
      :mode="authMode"
      @update:mode="authMode = $event"
      @submit="handleSubmit"
    />
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { getServerAddr, setServerAddr } from '../network/gameStream';
import ServerAddressInput from '../components/auth/ServerAddressInput.vue';
import AuthForm from '../components/auth/AuthForm.vue';

const emit = defineEmits(['login', 'register']);

const authMode = ref('login');
const serverAddr = ref(getServerAddr());

const onAddrChange = () => {
  const addr = serverAddr.value.trim();
  if (addr) setServerAddr(addr);
};

const handleSubmit = async ({ username, password }) => {
  if (!username || !password) return;
  if (authMode.value === 'login') {
    emit('login', username, password);
  } else {
    emit('register', username, password);
  }
};
</script>
