<template>
  <div class="about max-w-4xl mx-auto p-6">
    <h1 class="text-3xl font-bold mb-6">What are Liquidity Pools and How Do They Work?</h1>

    <!-- Tab buttons -->
    <div class="tabs flex flex-col sm:flex-row space-y-2 sm:space-y-0 sm:space-x-4 mb-6">
      <button
        v-for="(tab, idx) in tabList"
        :key="tab"
        @click="activeTab = idx"
        :class="[
          'px-4 py-2 rounded transition font-medium',
          activeTab === idx
            ? 'bg-accent text-bg'
            : 'bg-transparent text-accent hover:bg-accent/20'
        ]"
        :aria-selected="activeTab === idx"
        :aria-controls="`tab-panel-${idx}`"
        role="tab"
      >
        {{ tab }}
      </button>
    </div>

    <!-- Tab content -->
    <div class="tab-content">
      <div v-if="activeTab === 0" id="tab-panel-0" role="tabpanel">
        <h2 class="text-2xl font-semibold mb-4">Beginner: Understanding Liquidity Pools</h2>

        <h3 class="text-xl font-medium mb-2">Introduction to Liquidity Provision</h3>
        <p class="mb-4">
          <strong>Decentralized Finance (DeFi)</strong> is built on liquidity, it is the fuel that keeps decentralized exchanges (DEXs) running. When you provide liquidity, you’re not just parking tokens. You’re becoming part of the market infrastructure itself.
        </p>

        <h3 class="text-xl font-medium mb-2">What Is a Liquidity Pool (LP)?</h3>
        <p class="mb-4">
          A <strong>liquidity pool</strong> is a smart contract that holds two or more tokens so traders can swap between them without the need for a centralized intermediary like a centralized exchange (CEX). Think of it as a shared vault that everyone can trade against. When you deposit tokens into the vault, you earn a share of the trading fees, and sometimes additional protocol rewards. The more trades your pool handles, the more fees you collect. But it’s not risk-free, token prices move, and that’s where things get interesting.
        </p>

        <h3 class="text-xl font-medium mb-2">Key Terms You’ll See in PoolKeeper</h3>
        <ul class="list-disc pl-6 mb-4">
          <li>
            <strong>TVL (Total Value Locked)</strong>: How much value is sitting inside the pool. It’s a good proxy for pool size, activity, and confidence. Higher TVL = more stability, but not always more profit.
          </li>
          <li>
            <strong>Range (in Uniswap v3 and similar models)</strong>: Instead of providing liquidity across all prices, you choose a price range, a zone in which your liquidity is active. When the market price moves outside your range, your liquidity “goes out of range,” and you stop earning fees until you rebalance or the price moves back into range.
          </li>
          <li>
            <strong>APR / APY</strong>: Annual Percentage Rate / Annual Percentage Yield. APR shows raw yield from fees and rewards; APY assumes reinvestment of earnings (compounding).
          </li>
          <li>
            <strong>Impermanent Loss (IL)</strong>: The Big One! When you provide liquidity, you deposit two tokens into a pool. If their prices shift relative to each other, your position can lose value compared to simply holding those tokens in your wallet. Here's why: The pool automatically rebalances to maintain a ratio. When one token pumps, the pool sells some of it. When one dumps, the pool buys more. You end up with more of the losing token and less of the winning token. Why "impermanent"? If prices return to exactly where they started, the loss vanishes. But in reality, prices rarely return to the original ratio, making the loss permanent. Bottom line: You can earn fees in an LP, but if price divergence is too large, impermanent loss can wipe out those gains and leave you worse off than just hodling.
          </li>
          <li>
            <strong>Volatility</strong>: How much a token's price swings over time. High volatility = greater impermanent loss risk and requires a wider price range to stay in range longer. Low volatility = smaller price swings and allows for tighter, more capital-efficient ranges.
          </li>
          <li>
            <strong>Turnover Ratio (Volume/TVL)</strong>: Trading volume divided by total pool liquidity. High turnover means the pool is "working hard" processing lots of trades relative to its size, generating more fees per dollar of liquidity you've deployed.
          </li>
          <li>
            <strong>Harvesting & Compounding</strong>: Harvesting is claiming your earned fees. Compounding is reinvesting those fees back into the pool to increase your position size and earn fees on fees. Compounding accelerates returns but costs gas each time.
          </li>
          <li>
            <strong>Adjusting Liquidity (Enter/Exit)</strong>: You can increase your position by adding more capital or exit partially/fully by withdrawing liquidity. Every adjustment incurs gas fees and may reset your fee accumulation, so timing matters.
          </li>
        </ul>

        <h3 class="text-xl font-medium mb-2">How LPs Actually Earn</h3>
        <p class="mb-4">
          LPs earn trading fees (a percentage of every swap) and sometimes incentives (token rewards from the protocol). PoolKeeper helps track both sources and estimate what’s really driving your returns.
        </p>
        <p class="mb-4">
          <strong>Example</strong>: You deposit 1 ETH + 3,500 USDC into a pool. Fee: every time someone trades ETH↔USDC, you earn a share of the fee for providing the service. Reward: the contract creator can add incentives to boost allocation to the specific pool. Over time, if ETH price rises or falls, your token ratio shifts. You might have less ETH and more USDC or vice versa.
        </p>

        <h3 class="text-xl font-medium mb-2">The Role of Active Management</h3>
        <p class="mb-4">
          In concentrated-liquidity protocols, being active matters. You can:
        </p>
        <ul class="list-disc pl-6 mb-4">
          <li>Rebalance your range when price moves.</li>
          <li>Reinvest your rewards (compounding).</li>
          <li>Exit when volatility spikes.</li>
        </ul>
        <p class="mb-4">
          PoolKeeper automates or assists with these so you don’t need to babysit charts or write scripts.
        </p>

        <h3 class="text-xl font-medium mb-2">Risk and Strategy</h3>
        <p class="mb-4">
          DeFi isn’t a casino, it’s a math problem. Smart providers think in terms of risk buckets:
        </p>
        <ul class="list-disc pl-6 mb-4">
          <li><strong>Stablecoin pools</strong> (low risk, low reward)</li>
          <li><strong>Blue-chip pairs</strong> like ETH/USDC (moderate risk, good turnover)</li>
          <li><strong>Exotic pairs or new tokens</strong> (high risk, sometimes high reward)</li>
        </ul>
        <p class="mb-4">
          PoolKeeper scores pools across dimensions, helping you pick according to your tolerance, not someone else’s hype.
        </p>

        <h3 class="text-xl font-medium mb-2">The Bigger Picture</h3>
        <p class="mb-4">
          Liquidity provision is what keeps decentralized markets decentralized. It is an underlying service provided to the market enhancing decentralised infrastructure. You as a liquidity provider adds resilience, making sure trades can happen without banks, brokers, or middlemen. Whether you’re managing $100 or $1M, understanding how liquidity works is your edge and Poolkeeper is your tool to keep you on top of things.
        </p>
      </div>

      <div v-else-if="activeTab === 1" id="tab-panel-1" role="tabpanel">
        <h2 class="text-2xl font-semibold mb-4">Intermediate</h2>
        <p>Content for **Intermediate** goes here.</p>
      </div>

      <div v-else-if="activeTab === 2" id="tab-panel-2" role="tabpanel">
        <h2 class="text-2xl font-semibold mb-4">Advanced</h2>
        <p>Content for **Advanced** goes here.</p>
      </div>
    </div>
    <!-- Next Steps -->
    <!-- <div class="mt-6">
      <h3 class="text-xl font-semibold mb-4">Next Steps</h3>
      <ul class="list-disc pl-6">
        <li><a href="#" class="text-accent hover:underline">Explore Pools</a> – See real-time data and risk scores.</li>
        <li><a href="#" class="text-accent hover:underline">Learn LP Strategies</a> – Coming soon: tutorials and live examples.</li>
        <li><a href="#" class="text-accent hover:underline">Join the Community</a> – Talk strategy, share insights, and shape PoolKeeper’s roadmap.</li>
      </ul>
    </div> -->
  </div>
</template>

<script lang="ts" setup>
import { ref } from 'vue'

const tabList = ['Beginner', 'Intermediate', 'Advanced']
const activeTab = ref(0)
</script>

<style scoped>
.bg-accent {
  background-color: var(--color-accent, #3b82f6); /* Fallback to blue if not defined */
}
.text-accent {
  color: var(--color-accent, #3b82f6); /* Fallback to blue if not defined */
}
.text-bg {
  color: var(--color-bg, #ffffff); /* Fallback to white if not defined */
}
</style>