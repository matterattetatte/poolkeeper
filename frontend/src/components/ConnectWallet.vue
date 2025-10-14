<template>
  <div style="margin-left:auto">
    <wallet-multi-button dark></wallet-multi-button>
    <div v-if="wallet?.connected" class="">
      <p>Connected Wallet: {{ wallet.publicKey.toString().slice(0, 8) }}...</p>
      <button
      class="inline-block p-4 mt-6 text-center font-bold text-white bg-gradient-to-r from-[var(--color-accent)] to-[var(--color-accent-gradient)] border-2 border-[var(--color-border)] rounded-lg shadow-lg transform rotate-12"
       @click="disconnect">Disconnect</button>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { WalletMultiButton, useWallet } from 'solana-wallets-vue';

const { wallet, disconnect } = useWallet();
</script>

<style scoped>
button {
  background-color: #4caf50;
  color: white;
  padding: 10px;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  margin-top: 10px;
}
button:hover {
  background-color: #45a049;
}
</style>