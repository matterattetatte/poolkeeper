import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { WalletProvider } from '@solana/wallet-adapter-vue';
import { PhantomWalletAdapter, SolflareWalletAdapter } from '@solana/wallet-adapter-wallets';
import { clusterApiUrl } from '@solana/web3.js';

const network = WalletAdapterNetwork.Devnet; // Change to 'mainnet-beta' for production
const endpoint = clusterApiUrl(network);
const wallets = [
  new PhantomWalletAdapter(),
  new SolflareWalletAdapter({ network }),
  // Add more, e.g., new BackpackWalletAdapter()
];

const app = createApp(App);
app.use(router);
app.use(WalletProvider, { wallets, endpoint, autoConnect: true });
app.mount('#app');