<script lang="ts">
	import { Check } from '@lucide/svelte';
	import type { LoadedImage } from 'src/helpers/utils';

	let {
		active = $bindable(),
		image = $bindable(),
		onclick = () => {},
	}: {
		active: boolean;
		image: LoadedImage;
		onclick: () => void;
	} = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class={`_loaded-image hover:border-secondary-foreground relative cursor-pointer rounded border-2 p-2 transition-all ${active ? 'border-primary!' : ''}`}
	onclick={onclick}
>
	{#if image.completed}
		<div class="absolute top-2 right-2">
			<Check></Check>
		</div>
	{/if}
	<img class="aspect-square w-full object-contain" src={image.imgUrl} alt={image.name} />
	<div class="_loaded-image_text absolute bottom-2 w-full text-center opacity-0 transition-opacity">{image.name}</div>
</div>

<style>
	._loaded-image:hover {
		._loaded-image_text {
			opacity: 1;
		}
	}
</style>
