<script lang="ts">
	let { score }: { score: number } = $props();
	let string = $derived(
		score.toLocaleString('en-US', {
			minimumIntegerDigits: 4,
			useGrouping: false
		})
	);
</script>

<div class="container">
	{string}
</div>

<style>
	.container {
		display: flex;
		gap: 5px;
		justify-content: flex-end;
		padding-right: 20px;
	}
</style>
