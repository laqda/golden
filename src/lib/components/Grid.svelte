<script lang="ts">
	import type { Grid } from '$lib/core/grid';
	import Cell from './Cell.svelte';

	let {
		grid,
		height,
		width
	}: {
		grid: Grid;
		height: number;
		width: number;
	} = $props();
</script>

<div id="grid" style="--rows: {height}; --columns: {width}">
	{#each { length: width }, column}
		{#each { length: height }, row}
			<Cell x={column} y={row} cell={grid[column][row]} />
		{/each}
	{/each}
</div>

<style>
	:root {
		--border: 1px solid var(--color-primary);
		--cell-size: calc(200px / 8);
	}

	#grid {
		display: grid;
		grid-template-rows: repeat(8, 1fr);
		grid-template-columns: repeat(8, 1fr);

		border-top: var(--border);
		border-left: var(--border);

		& > :global(.cell) {
			border-bottom: var(--border);
			border-right: var(--border);
		}
	}
</style>
