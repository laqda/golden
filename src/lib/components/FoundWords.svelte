<script lang="ts">
	import type { FoundWord } from '$lib/wasm/golden_core';

	let { foundWords }: { foundWords: FoundWord[] } = $props();
</script>

<div id="foundWords">
	{#each foundWords as foundWord}
		<div class="foundWord">
			<span class="word">{foundWord.word()}</span>
			<span class="score">{foundWord.score()}</span>
		</div>
	{/each}
</div>

<style>
	#foundWords::-webkit-scrollbar {
		background-color: #fff;
		width: 16px;
		-webkit-appearance: none;
	}

	#foundWords::-webkit-scrollbar-track {
		background-color: #fff;
	}

	#foundWords::-webkit-scrollbar-thumb {
		background-color: #babac0;
		border-radius: 16px;
		border: 4px solid #fff;
	}

	#foundWords::-webkit-scrollbar-button {
		display: none;
	}

	#foundWords {
		display: flex;
		flex-direction: column;
		gap: 5px;

		padding-right: 5px;
		overflow-y: scroll;

		color: gray;
		font-size: 16px;
	}

	.foundWord {
		display: flex;
	}

	.score {
		margin-left: auto;
	}
</style>
