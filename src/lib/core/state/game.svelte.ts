import { type Game as WasmGame } from '$lib/wasm/golden_core';
import { createClockState } from './clock.svelte';
import { createGridState } from './grid.svelte';
import { createScoreState } from './score.svelte';
import { createTripletsState } from './triplets.svelte';
import { createFoundWordsState } from './foundWords.svelte';
import { transformToGrid } from '$lib/core/grid';
import { createGoldenState } from './golden.svelte';
import { initLettersTableContext } from './letterTable.svelte';
import { createUserEventsState } from './userEvents.svelte';

export function createGameState(wasmGame: WasmGame) {
	initLettersTableContext(wasmGame.get_letters_table());

	const clock = createClockState(wasmGame.clock_max_ms);
	const grid = createGridState();
	const score = createScoreState();
	const triplets = createTripletsState([...wasmGame.triplets()]);
	const foundWords = createFoundWordsState();
	const golden = createGoldenState([...wasmGame.golden_word()], wasmGame.golden_word_score);
	const userEvents = createUserEventsState();

	let lastFrame = 0;

	function tick(now: number) {
		if (lastFrame === 0) {
			lastFrame = now;
		}

		const delta = now - lastFrame;
		lastFrame = now;

		const snapshot = wasmGame.tick(delta, userEvents.extractUserClicks(), userEvents.lastHovered);

		// Update all state modules
		clock.update(snapshot.clock_remaining_ms);
		grid.update(transformToGrid(snapshot.grid()));
		score.update(wasmGame.score);
		triplets.updateIndex(wasmGame.triplets_current_index);
		foundWords.update(snapshot.found_words());

		requestAnimationFrame(tick);
	}

	// Start game loop
	requestAnimationFrame(tick);

	return {
		clock,
		grid,
		score,
		triplets,
		foundWords,
		golden
	};
}

export type GameState = ReturnType<typeof createGameState>;
