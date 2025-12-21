<script lang="ts">
	import Letter from './Letter.svelte';
	import { Cell as WasmCell, CellPathingStatus, Position } from '$lib/wasm/golden_core';
	import { getLetter } from '$lib/core/state/letterTable.svelte';
	import { useUserEvents } from '$lib/core/state/userEvents.svelte';

	let {
		x,
		y,
		cell
	}: {
		x: number;
		y: number;
		cell: WasmCell;
	} = $props();
	let letter = $derived(getLetter(cell.letter));
	let pathing_status = $derived(cell.pathing_status);

	const { click, hover } = useUserEvents();

	function onclick(_e: MouseEvent) {
		click(new Position(x, y));
	}

	function onmouseover(_e: MouseEvent) {
		hover(new Position(x, y));
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_mouse_events_have_key_events -->
<div
	class="cell"
	class:hasLetter={!!letter}
	class:blocked={pathing_status === CellPathingStatus.Blocked}
	class:path={pathing_status === CellPathingStatus.Path}
	{onclick}
	{onmouseover}
	style:grid-row={y + 1}
	style:grid-column={x + 1}
>
	{#if letter}
		<Letter {letter} />
	{/if}
</div>

<style>
	.cell {
		display: flex;
		justify-content: center;
		align-items: center;

		aspect-ratio: 1;
	}

	.cell > :global(*) {
		width: calc(100% - 10px);
		height: calc(100% - 10px);
		font-size: 30px;
	}

	.cell {
		background-color: white;
	}

	.cell.blocked {
		background-color: rgb(223, 223, 223);
	}

	.cell.path {
		background-color: var(--grid-path-color-primary);
	}

	.cell.hasLetter {
		cursor: pointer;
	}
</style>
