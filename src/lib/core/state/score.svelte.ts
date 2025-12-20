export function createScoreState() {
	let current = $state(0);

	return {
		get current() {
			return current;
		},
		update(newScore: number) {
			current = newScore;
		}
	};
}

export type ScoreState = ReturnType<typeof createScoreState>;
