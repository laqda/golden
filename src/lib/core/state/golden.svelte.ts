import { getContext, setContext } from 'svelte';

const GOLDEN_CONTEXT_KEY = Symbol('golden_context');

export interface GoldenContext {
	isGolden: (letter: number) => boolean;
}

export function createGoldenState(golden: number[], goldenWordScore: number) {
	const isGolden = (letter: number) => {
		return golden.includes(letter);
	};

	setContext<GoldenContext>(GOLDEN_CONTEXT_KEY, {
		isGolden
	});

	return {
		get golden() {
			return golden;
		},
		get goldenWordScore() {
			return goldenWordScore;
		},
		isGolden
	};
}

export type GoldenState = ReturnType<typeof createGoldenState>;

export function isGolden(letter: number): boolean {
	const context = getContext<GoldenContext>(GOLDEN_CONTEXT_KEY);
	return context.isGolden(letter);
}
