<script lang="ts">
	import type { ImageDimensions } from '../../helpers/image/imageWorkerRPC';
	import { Paintable } from '../../helpers/paintable/paintable';
	import { onMount } from 'svelte';

	let {
		paintable = $bindable(),
		dimensions = $bindable(),
	}: {
		paintable: Paintable | undefined;
		dimensions: ImageDimensions;
	} = $props();

	let canvas = $state<HTMLCanvasElement | null>(null);

	onMount(() => {
		if (!canvas) {
			console.error('Missing canvas element.');
			return;
		}
		paintable = new Paintable(canvas, dimensions, 'lime');
	});

	$effect(() => {
		if (paintable && dimensions) {
			paintable.clear(dimensions);
		}
	});
</script>

<canvas class="absolute top-0 right-0 bottom-0 left-0 h-full w-full opacity-15" bind:this={canvas}></canvas>
