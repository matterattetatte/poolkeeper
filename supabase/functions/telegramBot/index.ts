import { serve } from 'https://deno.land/std@0.201.0/http/server.ts';
import { Connection, PublicKey } from '@solana/web3.js';
import { getOrcaPools, getRaydiumPools } from './dex-clients.ts';
import { calculatePositionRange, isOutOfRange } from './position-utils.ts';

// 🔑 CONFIG - Replace with YOUR values
const CONFIG = {
  TELEGRAM_BOT_TOKEN: Deno.env.get('TELEGRAM_BOT_TOKEN')!,
  TELEGRAM_CHAT_ID: Deno.env.get('TELEGRAM_CHAT_ID')!,
  SOLANA_RPC: 'https://api.mainnet-beta.solana.com',
  // YOUR LP POSITIONS
  POSITIONS: [
    {
      poolAddress: '3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv', // WSOL/USDC Raydium
      positionAddress: 'YOUR_POSITION_PUBKEY_HERE', // ← ADD YOUR POSITION
      owner: 'YOUR_WALLET_PUBKEY_HERE',
      minPrice: 150,   // Notify if price < $150
      maxPrice: 200,   // Notify if price > $200
    },
    // Add more positions...
  ]
} as const;

const connection = new Connection(CONFIG.SOLANA_RPC);

// 📱 Send Telegram notification
async function sendTelegramAlert(message: string) {
  const url = `https://api.telegram.org/bot${CONFIG.TELEGRAM_BOT_TOKEN}/sendMessage`;
  await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      chat_id: CONFIG.TELEGRAM_CHAT_ID,
      text: `🚨 **LP OUT OF RANGE!**\n\n${message}`,
      parse_mode: 'Markdown'
    })
  });
}

// 🔍 Check single position
async function checkPosition(position: typeof CONFIG.POSITIONS[0]) {
  try {
    let poolInfo;
    
    // Get pool data based on pool address
    if (position.poolAddress === '3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv') {
      poolInfo = await getOrcaPools(position.poolAddress); // Raydium
    } else {
      poolInfo = await getRaydiumPools(position.poolAddress);
    }

    const { currentPrice, positionRange } = await calculatePositionRange(
      poolInfo,
      position.positionAddress
    );

    const outOfRange = isOutOfRange(currentPrice, position.minPrice, position.maxPrice);
    
    if (outOfRange) {
      const message = `
**${position.poolAddress.slice(0, 8)}...**
💰 **Current Price**: \$${currentPrice.toFixed(4)}
📊 **Your Range**: $${position.minPrice} - $${position.maxPrice}
🔴 **STATUS**: *OUT OF RANGE!*

**Action Required**: Add/Remove liquidity or wait for price recovery
      `;
      
      await sendTelegramAlert(message);
      console.log(`🚨 Alert sent for ${position.poolAddress}`);
    } else {
      console.log(`✅ ${position.poolAddress.slice(0, 8)}... IN RANGE`);
    }
  } catch (error) {
    console.error(`❌ Error checking ${position.poolAddress}:`, error);
  }
}

// 🚀 Main monitoring function
async function monitorPositions() {
  console.log('🔍 Checking LP positions...');
  
  for (const position of CONFIG.POSITIONS) {
    await checkPosition(position);
  }
  
  console.log('✅ Check complete!');
}

// 🌐 Deno Deploy HTTP Server
serve(async (req) => {
  if (req.method === 'OPTIONS') {
    return new Response('ok', { 
      headers: { 'Access-Control-Allow-Origin': '*' }
    });
  }

  try {
    await monitorPositions();
    return new Response('✅ Positions monitored!', { status: 200 });
  } catch (error) {
    console.error('❌ Bot error:', error);
    return new Response('❌ Error', { status: 500 });
  }
});