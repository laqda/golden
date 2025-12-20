<script lang="ts">
	import { isGolden } from '$lib/core/state/golden.svelte';
	import type { Letter } from '$lib/core/state/letterTable.svelte';

	let {
		letter,
		disabled = false
	}: {
		letter: Letter;
		disabled?: boolean;
	} = $props();

	let golden = $derived(isGolden(letter.index));

	// TODO display score
</script>

<div class="letter" class:disabled class:golden>
	{letter.char}
</div>

<style>
	.letter {
		width: 30px;
		height: 30px;

		background-color: var(--color-secondary);
		color: var(--color-primary);

		border: 1px solid var(--color-primary);
		border-radius: 5px;

		display: flex;
		justify-content: center;
		align-items: center;

		font-size: 20px;

		&.disabled {
			background-color: whitesmoke;
			border-color: lightgrey;
			color: gray;
		}

		&.golden {
			background-color: #ffd700;
			border-color: #796600;
			color: #625300;
		}
	}
</style>
