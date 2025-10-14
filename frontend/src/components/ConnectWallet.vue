<script setup lang="ts">
import { useWallet, WalletMultiButton } from '@solana/wallet-adapter-vue';
import { computed } from 'vue';

// Wallet state from the adapter
const { connected, publicKey, disconnect } = useWallet();

// Shorten the public key for display (e.g., "abcd...wxyz")
const shortAddress = computed(() => {
  if (!publicKey) return '';
  const pubkeyStr = publicKey.toBase58();
  return `${pubkeyStr.slice(0, 4)}...${pubkeyStr.slice(-4)}`;
});
</script>

<template>
  <div class="connect-wallet">
    <!-- WalletMultiButton handles the UI: Connect button, selector, and address display -->
    <WalletMultiButton />

    <!-- Optional: Custom connected state overlay (if you want more control) -->
    <div v-if="connected" class="connected-details">
      <span>{{ shortAddress }}</span>
      <button @click="disconnect" class="disconnect-btn">Disconnect</button>
    </div>
  </div>
</template>

<style scoped>
.connect-wallet {
  margin-left: auto; /* Pushes to the right in a flex nav */
  display: flex;
  align-items: center;
}

.connected-details {
  margin-left: 0.5rem;
  font-size: 0.8rem;
  display: flex;
  align-items: center;
}

.disconnect-btn {
  margin-left: 0.5rem;
  background: transparent;
  border: 1px solid var(--color-border);
  padding: 0.2rem 0.5rem;
  cursor: pointer;
  font-size: 0.8rem;
}

/* Style the adapter's button to match your nav */
:deep(.wallet-adapter-button) {
  background: transparent;
  border: 1px solid var(--color-border);
  padding: 0.5rem 1rem;
  cursor: pointer;
  font-size: inherit;
}
</style>