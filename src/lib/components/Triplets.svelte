<script lang="ts">
	import { getLetter } from '$lib/core/state/letterTable.svelte';
	import type { Triplet } from '$lib/context/triplets.svelte';
	import Letter from './Letter.svelte';

	let { next, future }: { next: Triplet; future: Triplet[] } = $props();

	let triplets = $derived(
		[next, ...future].map((triplet) => [
			getLetter(triplet.letter1)!,
			getLetter(triplet.letter2)!,
			getLetter(triplet.letter3)!
		])
	);
</script>

<div id="triplets">
	{#each triplets as triplet, i}
		<div class="triplet">
			{#each triplet as letter}
				<Letter {letter} disabled={i !== 0} />
			{/each}
		</div>
	{/each}
</div>

<style>
	/* total width */
	#triplets::-webkit-scrollbar {
		background-color: #fff;
		width: 16px;
	}

	/* background of the scrollbar except button or resizer */
	#triplets::-webkit-scrollbar-track {
		background-color: #fff;
	}

	/* scrollbar itself */
	#triplets::-webkit-scrollbar-thumb {
		background-color: #babac0;
		border-radius: 16px;
		border: 4px solid #fff;
	}

	/* set button(top and bottom of the scrollbar) */
	#triplets::-webkit-scrollbar-button {
		display: none;
	}

	#triplets {
		display: flex;
		flex-direction: column;
		gap: 5px;

		padding-right: 5px;
		overflow-y: scroll;
	}

	.triplet {
		display: flex;
		gap: 5px;
	}
</style>
