<script lang="ts">
	import { Game } from '$lib/wasm/golden_core';
	import GridComponent from '$lib/components/Grid.svelte';
	import Timer from '$lib/components/Timer.svelte';
	import RemainingLettersComponent from '$lib/components/RemainingLetters.svelte';
	import GoldenWordComponent from '$lib/components/GoldenWord.svelte';
	import Score from '$lib/components/Score.svelte';
	import TripletsComponent from '$lib/components/Triplets.svelte';
	import FoundWordsComponent from '$lib/components/FoundWords.svelte';
	import { createGameState } from '$lib/core/state/game.svelte';

	let {
		game
	}: {
		game: Game;
	} = $props();

	// Create unified game state
	const state = createGameState(game);
</script>

<div class="container">
	<div class="timer">
		<Timer cur={state.clock.remaining} max={state.clock.max} />
	</div>

	<div class="remaining-letters">
		<RemainingLettersComponent count={state.triplets.numberOfRemainingLetters} />
	</div>
	<div class="golden-word">
		<GoldenWordComponent
			goldenWord={state.golden.golden}
			goldenWordScore={state.golden.goldenWordScore}
		/>
	</div>
	<div class="score"><Score score={state.score.current} /></div>
	<TripletsComponent next={state.triplets.next} future={state.triplets.future} />

	{#if state.grid.grid}
		<div class="grid-container">
			<div class="grid-square">
				<GridComponent grid={state.grid.grid} width={game.grid_width} height={game.grid_height} />
			</div>
		</div>
	{/if}
	<FoundWordsComponent foundWords={state.foundWords.words} />
	<div class="bottom-menu">
		<!-- <button type="button" onclick={() => console.log('TODO')}>New Game</button> -->
	</div>
</div>

<style>
	:root {
		--grid-size: 640px;

		--color-primary: color(srgb 0.0758 0.3382 0.37);
		--color-secondary: color(srgb 0.7859 0.9148 0.93);

		--grid-path-color-primary: color(srgb 0.0758 0.3382 0.37);
	}

	:global(html) {
		overscroll-behavior: none;
		font-family: 'arial';
	}

	:global(html),
	:global(body) {
		height: 100vh;
		margin: 0;

		display: flex;
		align-items: center;
	}

	.container {
		user-select: none;
		-webkit-user-select: none;

		margin: 0;
		min-width: fit-content;

		padding: 10px;
		box-sizing: border-box;

		display: grid;
		grid-gap: 10px;
		grid-template-columns:
			[timer] 10px [triplets] 127px [grid] var(--grid-size)
			[found_words] 160px 10px [end];
		grid-template-rows: [top-menu] 45px [middle] var(--grid-size) [bottom-menu] 40px [end];
	}

	.timer {
		grid-column: timer;
		grid-row: middle / bottom-menu;
	}

	.remaining-letters {
		grid-column: triplets;
		grid-row: top-menu;
	}

	.golden-word {
		grid-column: grid;
		grid-row: top-menu;
	}

	.score {
		grid-column: found_words;
		grid-row: top-menu;
	}

	.bottom-menu {
		grid-column: timer / end;
		grid-row: bottom-menu;
		display: flex;
	}

	.container > :global(#triplets) {
		grid-column: triplets;
		grid-row: middle;
	}

	.container > :global(#foundWords) {
		grid-column: found_words;
		grid-row: middle;
	}

	.container > .grid-container {
		grid-column: grid;
		grid-row: middle;
	}

	.grid-square {
		aspect-ratio: 1;
		max-height: var(--grid-size);
	}
</style>
