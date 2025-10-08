<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { getImageDimensions, removeExtension, type LoadedImage } from 'src/helpers/utils';
	import LoadedImageCard from './LoadedImageCard.svelte';
	import { onDestroy, untrack } from 'svelte';
	import Button from './ui/button/button.svelte';
	import type { ImageProcessor } from 'src/helpers/image/imageProcessor';
	import JSZip from 'jszip';
	import { Progress } from '$lib/components/ui/progress/index.js';
	import Checkbox from './ui/checkbox/checkbox.svelte';
	import { Label } from '$lib/components/ui/label/index.js';
	import type { ImageRenderOptions } from 'image-processing/pkg/image_processing';

	interface ExportOptions {
		subject: boolean;
		ring: boolean;
	}

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
	let exportOptions = $state<ExportOptions>({
		subject: true,
		ring: true,
	});
	let completedRenders = $state<number>(0);
	let failedRenders = $state<number>(0);
	let zipUrl = $state<string | null>(null);

	let toExportCount = $derived(
		(exportOptions.subject ? selectedImages.length : 0) + (exportOptions.ring ? selectedImages.length : 0),
	);

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

		clearDownload();

		mode = 'export';
		completedRenders = 0;
		failedRenders = 0;

		const jszip = new JSZip();

		await Promise.allSettled(
			selectedImages.map(async index => {
				const image = images[index];
				const imageName = removeExtension(image.name);

				const imageMask = image.mask ? new Uint8Array(image.mask.data) : undefined;
				const imageDimensions = getImageDimensions(image.size, image.oversized);
				const imageTransform = $state.snapshot(image.transform);

				if (exportOptions.subject) {
					try {
						const opts: ImageRenderOptions = {
							dimensions: imageDimensions,
							transform: imageTransform,
							ring: false,
						};

						const data = await imageProcessor.render(image.data, imageMask, opts);
						completedRenders += 1;

						jszip.file(`${imageName}-subject.webp`, data);
					} catch (error) {
						console.warn(`Error rendering image ${image.name}-subject:`, error);
						failedRenders += 1;
					}
				}

				if (exportOptions.ring) {
					try {
						const opts: ImageRenderOptions = {
							dimensions: imageDimensions,
							transform: imageTransform,
							ring: true,
						};

						const data = await imageProcessor.render(image.data, imageMask, opts);
						completedRenders += 1;

						jszip.file(`${imageName}-token.webp`, data);
					} catch (error) {
						console.warn(`Error rendering image ${image.name}-token:`, error);
						failedRenders += 1;
					}
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
		clearDownload();
	});

	function clearDownload() {
		if (zipUrl) {
			URL.revokeObjectURL(zipUrl);
			zipUrl = null;
		}
	}
</script>

<Dialog.Root bind:open={open}>
	<Dialog.Content class="w-[80vw]! max-w-[80vw]!">
		{#if mode === 'select'}
			<div class="flex flex-col gap-2">
				<div class="text-lg leading-none font-semibold">Export options</div>
				<div class="text-muted-foreground text-sm">
					Select how your tokens should be exported and which token versions the export should include.
				</div>
			</div>

			<div class="flex flex-col gap-6">
				<div class="flex items-center gap-3">
					<Checkbox bind:checked={exportOptions.subject} />
					<Label>Include version with <strong>no</strong> token ring (for use with dynamic token rings)</Label
					>
				</div>
				<div class="flex items-center gap-3">
					<Checkbox bind:checked={exportOptions.ring} />
					<Label>Include version with token ring</Label>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<div class="text-lg leading-none font-semibold">Image selection</div>
				<div class="text-muted-foreground text-sm">
					Select which images you want to export as tokens. You can select images via the buttons below or by
					clicking on the images. The images you marked as completed while editing have a checkmark in the top
					right corner.
				</div>
			</div>

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
						selectedImages = images.map((_, index) => index).filter(index => images[index].completed);
					}}
				>
					Select completed
				</Button>

				<div class="flex flex-1 items-center justify-end">
					<span class="text-muted-foreground text-sm">
						Selected: {selectedImages.length} / {images.length}
					</span>
				</div>
				<Button variant="default" disabled={toExportCount === 0} onclick={() => exportSelectedImages()}>
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
			{#if !zipUrl}
				<div class="flex flex-col gap-2">
					<div class="text-lg leading-none font-semibold">Exporting tokens</div>
					<div class="text-muted-foreground text-sm">
						Exporting your tokens. This may take a while depending on the number of images you selected.
					</div>
				</div>
			{:else}
				<div class="flex flex-col gap-2">
					<div class="text-lg leading-none font-semibold">Export completed</div>
					<div class="text-muted-foreground text-sm">
						A <code>.zip</code> should have downloaded automatically.
						<a href={zipUrl}>If not, you can click here to retry the download.</a>
					</div>
				</div>
			{/if}

			<div class="flex flex-col gap-4">
				<div class="flex flex-col gap-2">
					{#if !zipUrl}
						<span class="text-muted-foreground text-sm"
							>Exporting {completedRenders} / {toExportCount} images...</span
						>
					{/if}
					{#if failedRenders > 0}
						<span class="text-sm text-red-500">
							{failedRenders} images failed to render.
						</span>
					{/if}
					<Progress value={(completedRenders / toExportCount) * 100} />
				</div>
				{#if zipUrl}
					<div class="flex flex-row-reverse gap-2">
						<Button variant="default" onclick={() => (mode = 'select')}>Back to selection</Button>
					</div>
				{/if}
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>
