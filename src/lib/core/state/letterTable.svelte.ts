import { getContext, setContext } from 'svelte';
import { is_empty_cell, LettersTable } from '$lib/wasm/golden_core';

const LETTERS_TABLE_CONTEXT_KEY = Symbol('letters_table_context');

export interface LettersTableContext {
	getLetter: (index: number) => string;
	getScore: (index: number) => number;
}

export function initLettersTableContext(lettersTable: LettersTable) {
	setContext<LettersTableContext>(LETTERS_TABLE_CONTEXT_KEY, {
		getLetter: (index: number) => lettersTable.try_get_letter_char(index),
		getScore: (index: number) => lettersTable.try_get_letter_score(index)
	});
}

export function getLetter(index: number): Letter | null {
	if (is_empty_cell(index)) {
		return null;
	}

	const context = getContext<LettersTableContext>(LETTERS_TABLE_CONTEXT_KEY);
	return { index, char: context.getLetter(index), score: context.getScore(index) };
}

export function getWord(letters: number[]): Letter[] {
	return letters.map((index) => {
		const letter = getLetter(index);
		if (!letter) {
			throw new Error(`Invalid letter index in word: ${index}`);
		}
		return letter;
	});
}

export interface Letter {
	index: number;
	char: string;
	score: number;
}
