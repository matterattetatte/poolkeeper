<script setup lang="ts">
import Logo from '@/components/Logo.vue';
import { onMounted } from 'vue'
import { sdk } from '@farcaster/miniapp-sdk'


onMounted(() => {
	try {
		sdk.actions.ready()
	} catch (e) {
		console.error('ready() failed', e)
	}
})

</script>

<template>
  <main>
    <h1 class="text-8xl text-center font-bold pt-10 w-full uppercase">
      Get <br /> Stress-free<br /> LP Yield
      <Logo class="inline-block pb-2 ml-4" />
    </h1>
    <p class="text-center mb-8 text-2xl">
      control panel for Solana yield
    </p>
  </main>
</template>

<style scoped>

img {
  height: 64px;
  margin-bottom: 1rem;
}

</style>
