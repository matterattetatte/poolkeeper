<!-- list 3 lps  -->

<script setup lang="ts">
import { RouterLink } from "vue-router";

const pools = [
    {
        address: '0x5e7BB104d84c7CB9B682AaC2F3d509f5F406809A',
        name: 'WETH / USDC',
        network: 'base',
        exchange: 'aerodrome',
        baseTokenAddress: '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913' // usdc
    },
    {
        address: '0x420DD381b31aEf6683db6B902084cB0FFECe40Da',
        name: 'WETH / cbBTC',
        network: 'base',
        exchange: 'aerodrome',
        baseTokenAddress: '0xcbb7c0000ab88b473b1f5afd9ef808440eed33bf' //cbBTC
    },
    {
        address: '0xd0b53D9277642d899DF5C87A3966A349A798F224',
        name: 'WETH / USDC',
        network: 'base',
        exchange: 'uniswap',
        baseTokenAddress: '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913' // usdc
    },
    {
        address: '3ucNos4NbumPLZNWztqGHNFFgkHeRMBQAVemeeomsUxv',
        name: 'WSOL-USDC (Raydium)',
        network: 'solana',
        exchange: 'raydium',
        baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v' // usdc
    },
    {
        address: 'Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE',
        name: 'WSOL/USDC (Orca)',
        network: 'solana',
        exchange: 'orca',
        baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v' // usdc
    },
    {
        address: '6R4r93V5fcMzc13CL2enEepDSYcr4Qx3ptZBDwudTXCo',
        name: 'NVDAx-USDC (Orca)',
        network: 'solana',
        exchange: 'orca',
        baseTokenAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'  // usdc
    },
]

</script>

<template>
  <main>
    <!-- each lp should be clickable and navigate to a new view
      -->
    <div class="container mx-auto p-4">
      <h1 class="text-2xl font-bold mb-4">Liquidity Pools</h1>
      <h2>Available Liquidity Pools</h2>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
 <div
    v-for="pool in pools"
    :key="pool.address"
    class="relative border p-4 rounded-lg transition-all duration-200"
    :class="{
      'opacity-50 cursor-not-allowed': pool.network === 'base',
      'hover:shadow-lg cursor-pointer': pool.network !== 'base'
    }"
  >
    <!-- Only render RouterLink if network is NOT base -->
    <RouterLink
      v-if="pool.network !== 'base'"
      :to="`/pools/${pool.address}?name=${pool.name}`"
      class="block"
    >
      <h3 class="text-xl font-semibold mb-2">{{ pool.name }}</h3>
      <p class="text-sm text-gray-600">
        Address: {{ pool.address.slice(0, 6) }}...{{ pool.address.slice(-4) }}
      </p>
    </RouterLink>

    <!-- Disabled version for Base network -->
    <div v-else class="block">
      <h3 class="text-xl font-semibold mb-2 text-gray-500">{{ pool.name }}</h3>
      <p class="text-sm text-gray-500">
        Address: {{ pool.address.slice(0, 6) }}...{{ pool.address.slice(-4) }}
      </p>
      
      <!-- Coming Soon Badge -->
      <span class="absolute top-3 right-3 bg-gray-200 text-gray-700 text-xs font-medium px-3 py-1 rounded-full">
        Coming Soon
      </span>
    </div>
  </div>
      </div>
    </div>

    <div class="container mx-auto p-4 mt-8">
      <h2 class="text-2xl font-bold mb-4">My Positions</h2>
      <p>You have no positions yet!</p>
    </div>

  </main>
</template>
