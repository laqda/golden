import type { Grid } from '$lib/core/grid';

export function createGridState() {
	let grid = $state<Grid | null>(null);

	return {
		get grid() {
			return grid;
		},
		update(newGrid: Grid | null) {
			grid = newGrid;
		}
	};
}

export type GridState = ReturnType<typeof createGridState>;
