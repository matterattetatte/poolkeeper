import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import SolanaWallets from 'solana-wallets-vue';
import 'solana-wallets-vue/styles.css'; // Default styles
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { PhantomWalletAdapter, SolflareWalletAdapter } from '@solana/wallet-adapter-wallets';

const walletOptions = {
  wallets: [
    new PhantomWalletAdapter(),
    new SolflareWalletAdapter({ network: WalletAdapterNetwork.Devnet }),
  ],
  autoConnect: false, // Manual connection for user control
};

const app = createApp(App)
app.use(SolanaWallets, walletOptions)
app.use(router) // Dynamic import for router
.mount('#app');