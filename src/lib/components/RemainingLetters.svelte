<script lang="ts">
	let { count }: { count: number } = $props();
	let string = $derived(
		count.toLocaleString('en-US', {
			minimumIntegerDigits: 3,
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
	}
</style>
