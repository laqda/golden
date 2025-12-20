<script lang="ts">
	let { cur, max }: { cur: number; max: number } = $props();
	let barPercentage = $derived((cur / max) * 100);
	let barHeight = $derived(`${barPercentage}%`);
</script>

<div id="timer">
	<div id="timer-bar" style:height={barHeight}></div>
</div>

<style>
	#timer {
		width: 100%;
		height: 100%;

		display: flex;
		flex-direction: column-reverse;

		background-color: rgb(236, 236, 236);
	}

	#timer-bar {
		width: 100%;

		background-color: #acacac;
	}
</style>
