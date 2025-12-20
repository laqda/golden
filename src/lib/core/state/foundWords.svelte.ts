import type { FoundWord } from '$lib/wasm/golden_core';

export function createFoundWordsState() {
	let words = $state<FoundWord[]>([]);

	return {
		get words() {
			return words;
		},
		update(newWords: FoundWord[]) {
			words = newWords;
		}
	};
}

export type FoundWordsState = ReturnType<typeof createFoundWordsState>;
