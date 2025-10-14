<!-- src/components/ConnectWallet.vue -->
<template>
  <div class="connect-wallet">
    <wallet-multi-button dark></wallet-multi-button>
    <div v-if="wallet?.connected" class="wallet-info">
      <p>Connected Wallet: {{ wallet.publicKey.toString().slice(0, 8) }}...</p>
      <button @click="disconnect">Disconnect</button>
    </div>
    <div v-else>
      <p>No wallet connected</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { WalletMultiButton, useWallet } from 'solana-wallets-vue';

const { wallet, disconnect } = useWallet();
</script>

<style scoped>
.connect-wallet {
  padding: 20px;
  max-width: 400px;
  margin: 0 auto;
  text-align: center;
}
.wallet-info {
  margin-top: 20px;
}
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