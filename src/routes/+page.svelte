<script lang="ts">
	import { onMount } from 'svelte';
	import init, { Game, init as wasmInit } from '$lib/wasm/golden_core';
	import GameComponent from '$lib/components/Game.svelte';
	import { seedOfTheDay } from '$lib/core/seed';

	let game: Game;

	const clock_ms = 20_000; // 20 seconds
	const width = 8;
	const height = 8;

	onMount(async () => {
		await init();
		wasmInit();

		const seed = seedOfTheDay();
		game = new Game(clock_ms, width, height, seed);
	});
</script>

<div class="page">
	{#if game}
		<GameComponent {game} />
	{/if}
</div>

<style>
	.page {
		display: flex;
		justify-content: center;
		align-items: center;

		min-height: 100vh;
		width: 100vw;
	}
</style>
