<template>
  <div class="relative inline-block text-left">
    <button
      class="inline-flex items-center justify-center p-4 mt-6 text-center font-bold text-white bg-[var(--color-accent-gradient)] border-2 border-gray-300 rounded-lg shadow-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition-all duration-200"
      @click="isOpen = !isOpen"
    >
      <Logo class="w-5 h-5 mr-2" />
      Drop into LP position...
    </button>

    <div
      v-if="isOpen"
      class="absolute mt-2 w-56 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 focus:outline-none z-10"
    >
      <div class="py-1">
        <a
          v-for="token in tokens"
          :key="token.symbol"
          href="#"
          class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900"
          @click.prevent="isOpen = false"
        >
          {{ token.tempText }}
        </a>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import Logo from '@/components/Logo.vue';
import { ref } from 'vue';

const isOpen = ref(false);

const tokens = [
  {
    name: 'Solana',
    symbol: 'SOL',
    tempText: '15 SOL ($314.40)',
  },
  {
    name: 'USD Coin',
    symbol: 'USDC',
    tempText: '121 USDC ($121.00)',
  },
];
</script>