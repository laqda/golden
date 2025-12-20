export function createClockState(maxMs: number) {
	const max = maxMs;
	let remaining = $state(maxMs);

	return {
		get max() {
			return max;
		},
		get remaining() {
			return remaining;
		},
		update(newRemaining: number) {
			remaining = newRemaining;
		}
	};
}

export type ClockState = ReturnType<typeof createClockState>;
