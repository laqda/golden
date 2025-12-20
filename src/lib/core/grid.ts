import { Cell as WasmCell } from '$lib/wasm/golden_core';

export function transformToGrid(raw: WasmCell[]): Grid {
	return raw.reduce<Grid>((cells: Grid, rawCell) => {
		const x = rawCell.position.x;
		const y = rawCell.position.y;

		if (!cells[x]) {
			cells[x] = {};
		}

		cells[x][y] = rawCell;

		return cells;
	}, {} as Grid);
}

export interface Position {
	x: number;
	y: number;
}

export interface Grid {
	[x: number]: { [y: number]: WasmCell };
}
