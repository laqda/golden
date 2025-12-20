<script lang="ts">
	import { getWord } from '$lib/core/state/letterTable.svelte';
	import Letter from './Letter.svelte';

	let { goldenWord, goldenWordScore }: { goldenWord: number[]; goldenWordScore: number } = $props();

	let scoreDigits = $derived(
		goldenWordScore.toLocaleString('en-US', {
			minimumIntegerDigits: 3,
			useGrouping: false
		})
	);

	let letters = $derived(getWord(goldenWord));
</script>

<div class="container">
	<div>
		{#each letters as letter}
			<Letter {letter} />
		{/each}
	</div>
	<div>
		{scoreDigits}
	</div>
</div>

<style>
	.container {
		display: flex;
		gap: 20px;
		justify-content: center;
	}

	.container > div {
		display: flex;
		gap: 5px;
	}
</style>
