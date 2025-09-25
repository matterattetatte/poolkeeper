<!-- list 3 lps  -->

<script setup lang="ts">
import { RouterLink } from "vue-router";

const lps = [
  { id: 1, platform: 'Orca', name: "SOL/USDC", liquidity: "$1,000,000", volume: "$500,000", address: "Czfq3xZZDmsdGdUyrNLtRhGc47cXcZtLG4crryfu44zE" },
  { id: 2, name: "Pool B", liquidity: "$750,000", volume: "$300,000", address: "0x456...def" },
  { id: 3, name: "Pool C", liquidity: "$500,000", volume: "$200,000", address: "0x789...ghi" },
];

</script>

<template>
  <main>
    <!-- each lp should be clickable and navigate to a new view
      -->
    <div class="container mx-auto p-4">
      <h1 class="text-2xl font-bold mb-4">Liquidity Pools</h1>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- display all in here -->
         <h2>Available Liquidity Pools</h2>
        <RouterLink :to="`/lps/${lp.address}`" v-for="lp in lps" :key="lp.id" class="border p-4 rounded-lg hover:shadow-lg cursor-pointer">
          <h3 class="text-xl font-semibold mb-2">{{ lp.name }}</h3>
          <p>Liquidity: {{ lp.liquidity }}</p>
          <p>Volume: {{ lp.volume }}</p>
          <p>Address: {{ lp.address }}</p>
        </RouterLink>
      </div>
    </div>
  </main>
</template>
