<!-- list 3 lps  -->

<script setup lang="ts">
import { RouterLink } from "vue-router";

const pools = [
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
        <!-- display all in here -->
        <RouterLink :to="`/lps/${pool.address}?name=${pool.name}`" v-for="pool in pools" :key="pool.address" class="border p-4 rounded-lg hover:shadow-lg cursor-pointer">
          <h3 class="text-xl font-semibold mb-2">{{ pool.name }}</h3>
          <p>Address: {{ pool.address.slice(0, 3) }}...{{ pool.address.slice(-3) }}</p>
        </RouterLink>
      </div>
    </div>

    <div class="container mx-auto p-4 mt-8">
      <h2 class="text-2xl font-bold mb-4">My Positions</h2>
      <p>You have no positions yet!</p>
    </div>

  </main>
</template>
