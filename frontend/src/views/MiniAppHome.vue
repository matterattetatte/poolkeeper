<template>
	<div class="miniapp">
		<h1>PoolKeeper Mini App</h1>
		<p>Quick actions inside Farcaster/Base App.</p>
		<div class="actions">
			<button @click="composeCast">Compose Cast</button>
			<button @click="openDocs">Open Docs</button>
		</div>
	</div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { sdk } from '@farcaster/miniapp-sdk'

onMounted(() => {
	try {
		sdk.actions.ready()
	} catch (e) {
		console.error('ready() failed', e)
	}
})

function composeCast() {
	try {
		sdk.actions.composeCast({
			text: 'Exploring PoolKeeper Mini App! #Farcaster',
			embeds: ['https://poolkeeper']
		})
	} catch (e) {
		console.error('composeCast failed', e)
	}
}

function openDocs() {
	try {
		sdk.actions.openUrl('https://docs.base.org/base-app/build-with-minikit/overview')
	} catch (e) {
		console.error('openUrl failed', e)
	}
}
</script>

<style scoped>
.miniapp {
	padding: 1.5rem;
}
.actions {
	display: flex;
	gap: 0.75rem;
	margin-top: 1rem;
}
button {
	padding: 0.5rem 0.9rem;
	border-radius: 8px;
	border: 1px solid #2b3653;
	background: #0b0f1a;
	color: #e6eefc;
	cursor: pointer;
}
button:hover {
	background: #12203a;
}
</style>
