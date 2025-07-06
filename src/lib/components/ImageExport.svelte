<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { getImageDimensions, type LoadedImage } from 'src/helpers/utils';
	import LoadedImageCard from './LoadedImageCard.svelte';
	import { onDestroy, untrack } from 'svelte';
	import Button from './ui/button/button.svelte';
	import type { ImageProcessor } from 'src/helpers/image/imageProcessor';
	import JSZip from 'jszip';
	import { Progress } from '$lib/components/ui/progress/index.js';

	let {
		open = $bindable(),
		images = $bindable(),
		imageProcessor,
	}: {
		open: boolean;
		images: LoadedImage[];
		imageProcessor: ImageProcessor;
	} = $props();

	let selectedImages = $state<number[]>([]);
	let mode = $state<'select' | 'export'>('select');
	let completedRenders = $state<number>(0);
	let failedRenders = $state<number>(0);
	let zipUrl = $state<string | null>(null);

	$effect(() => {
		let _ = images;
		untrack(() => {
			selectedImages = [];
		});
	});

	async function exportSelectedImages() {
		if (selectedImages.length === 0) {
			return;
		}

		mode = 'export';
		completedRenders = 0;
		failedRenders = 0;
		if (zipUrl) {
			URL.revokeObjectURL(zipUrl);
		}
		zipUrl = null;

		const jszip = new JSZip();

		await Promise.allSettled(
			selectedImages.map(async index => {
				const image = images[index];

				try {
					const data = await imageProcessor.asyncRender(
						image.data,
						image.mask ? new Uint8Array(image.mask.data) : undefined,
						getImageDimensions(image.size, image.oversized),
						$state.snapshot(image.transform),
						false,
					);
					completedRenders += 1;

					jszip.file(image.name, data);
				} catch (error) {
					console.warn(`Error rendering image ${image.name}:`, error);
					failedRenders += 1;
				}
			}),
		);

		const zip = await jszip.generateAsync({ type: 'blob' });
		zipUrl = URL.createObjectURL(zip);

		downloadZip();
	}

	function downloadZip() {
		if (!zipUrl) {
			console.warn('No zip file to download.');
			return;
		}

		const a = document.createElement('a');
		document.body.appendChild(a);
		a.style = 'display: none';

		a.href = zipUrl;
		a.download = `tokens.zip`;
		a.click();
	}

	onDestroy(() => {
		if (zipUrl) {
			URL.revokeObjectURL(zipUrl);
		}
	});
</script>

<Dialog.Root bind:open={open}>
	<Dialog.Content class="w-[80vw]! max-w-[80vw]!">
		<Dialog.Header>
			<Dialog.Title>Export Tokens</Dialog.Title>
			<Dialog.Description>Export your tokens as images.</Dialog.Description>
		</Dialog.Header>

		{#if mode === 'select'}
			<div class="flex flex-row gap-2">
				<Button
					variant="outline"
					onclick={() => {
						selectedImages = images.map((_, index) => index);
					}}
				>
					Select all
				</Button>
				<Button
					variant="outline"
					onclick={() => {
						selectedImages = [];
					}}
				>
					Unselect all
				</Button>
				<Button
					variant="outline"
					onclick={() => {
						selectedImages = images.filter(image => image.completed).map((_, index) => index);
					}}
				>
					Select completed
				</Button>
				<div class="flex flex-1 items-center justify-end">
					<span class="text-muted-foreground text-sm">
						Selected: {selectedImages.length} / {images.length}
					</span>
				</div>
				<Button variant="default" disabled={selectedImages.length === 0} onclick={() => exportSelectedImages()}>
					Export
				</Button>
			</div>

			<div class="grid max-h-[80vh] w-full grid-cols-3 gap-2 overflow-x-clip overflow-y-auto">
				{#each images as image, index}
					<LoadedImageCard
						active={selectedImages.includes(index)}
						image={image}
						onclick={() => {
							if (selectedImages.includes(index)) {
								selectedImages = selectedImages.filter(i => i !== index);
							} else {
								selectedImages.push(index);
							}
						}}
					></LoadedImageCard>
				{/each}
			</div>
		{:else}
			<div class="flex flex-col gap-4">
				<div class="flex flex-col gap-2">
					<Progress value={(completedRenders / selectedImages.length) * 100} />
					{#if zipUrl}
						<span>Export completed!</span>
					{:else}
						<span>Exporting {completedRenders} / {selectedImages.length} images...</span>
					{/if}
					{#if failedRenders > 0}
						<span class="text-sm text-red-500">
							{failedRenders} images failed to render.
						</span>
					{/if}
				</div>
				{#if zipUrl}
					<div class="flex flex-row gap-2">
						<a href={zipUrl}>Download again</a>
						<Button variant="default" onclick={() => (mode = 'select')}>Back to selection</Button>
					</div>
				{/if}
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
