<script lang="ts">
	import { CreatureSize, EditMode, getImageDimensions, readFileAsArrayBuffer, remapRange } from '../../helpers/utils';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { ImageProcessor } from '../../helpers/imageProcessor';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { tick, untrack } from 'svelte';
	import { Slider } from '$lib/components/ui/slider/index.js';
	import type { ImageTransform } from 'src/helpers/imageWorkerRPCConfig';
	import { Paintbrush, Scaling } from '@lucide/svelte';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import { Label } from '$lib/components/ui/label/index.js';

	interface LoadedImage {
		name: string;
		data: Uint8Array;
		mask: Uint8Array | undefined;
		size: CreatureSize;
		oversized: boolean;
		transform: ImageTransform;
	}

	const imageProcessor = new ImageProcessor();

	async function loadImages(): Promise<void> {
		if (!fileInput) {
			console.warn('File input element not found');
			return;
		}
		let files = fileInput.files;
		if (!files || files.length === 0) {
			console.warn('No files selected');
			return;
		}

		activeImageIndex = undefined;
		loadedImages = await Promise.all(
			Array.from(files).map(async file => {
				const data = await readFileAsArrayBuffer(file);
				return {
					name: file.name,
					data: new Uint8Array(data),
					mask: undefined,
					size: CreatureSize.Medium,
					oversized: false,
					transform: {
						scale: 1,
						posX: 0,
						posY: 0,
						flipped: false,
					},
				} satisfies LoadedImage;
			}),
		);
	}

	let loadedImages = $state<LoadedImage[]>([]);

	let activeImageIndex = $state<number | undefined>(undefined);
	let activeImage = $derived<LoadedImage | undefined>(activeImageIndex ? loadedImages[activeImageIndex] : undefined);
	let dimensions = $derived(
		activeImage
			? getImageDimensions(activeImage.size, activeImage.oversized)
			: getImageDimensions(CreatureSize.Medium, false),
	);
	let brushSize = $state<number>(40);
	let editMode = $state<EditMode>(EditMode.Positioning);

	let previewImg = $state<HTMLImageElement | undefined>(undefined);
	let maskImg = $state<HTMLImageElement | undefined>(undefined);
	let fileInput = $state<HTMLInputElement | null>();

	let canvasSize = $state<number>(0);
	let canvasSizeMul = $derived<number>(canvasSize > 0 ? canvasSize / dimensions.size : 1);

	function onDimsChange() {
		if (activeImage && maskImg) {
			let tmpImg = activeImage;
			let tmpMaskImage = maskImg;
			imageProcessor.clearMask(dimensions, data => {
				tmpImg.mask = data;
				tmpMaskImage.src = URL.createObjectURL(new Blob([data]));
			});
		}
	}

	async function setActiveImage(file: LoadedImage) {
		activeImage = file;
		await tick(); // Ensure the DOM is updated before rendering
		if (maskImg) {
			let tmpMaskImage = maskImg;
			if (file.mask) {
				console.log('loading mask for', file.name);

				tmpMaskImage.src = URL.createObjectURL(new Blob([file.mask]));
				imageProcessor.setMask(file.mask, dimensions);
			} else {
				imageProcessor.clearMask(dimensions, data => {
					file.mask = data;
					tmpMaskImage.src = URL.createObjectURL(new Blob([data]));
				});
			}
		}
	}

	function handleCanvasClick(event: MouseEvent) {
		if (!activeImage) {
			return; // No active image to edit
		}

		if (editMode === EditMode.Painting) {
			updateMask(event);
		}
	}

	function updateMask(event: MouseEvent) {
		if (!activeImage || !maskImg || editMode !== EditMode.Painting) {
			return;
		}

		const rect = maskImg.getBoundingClientRect();
		const x = event.clientX - rect.left;
		const y = event.clientY - rect.top;
		const imgX = remapRange(x, 0, rect.width, 0, dimensions.size);
		const imgY = remapRange(y, 0, rect.height, 0, dimensions.size);

		if (imgX < 0 || imgY < 0 || imgX > dimensions.size || imgY > dimensions.size) {
			return; // Click outside the image bounds
		}

		imageProcessor.updateMask(brushSize, event.button === 0, imgX, imgY, data => {
			if (!activeImage) return;

			activeImage.mask = data;
			if (maskImg) {
				maskImg.src = URL.createObjectURL(new Blob([data]));
			}
		});
	}

	let dragStartX = $state<number | null>(null);
	let dragStartY = $state<number | null>(null);
	let dragStartImgX = $state<number | null>(null);
	let dragStartImgY = $state<number | null>(null);

	function handleCanvasDragStart(event: DragEvent) {
		if (!activeImage || editMode !== EditMode.Positioning) {
			return; // Only allow dragging in positioning mode
		}

		dragStartX = event.clientX;
		dragStartY = event.clientY;
		dragStartImgX = activeImage.transform.posX;
		dragStartImgY = activeImage.transform.posY;
	}

	function handleCanvasDrag(event: DragEvent) {
		if (
			!activeImage ||
			dragStartX === null ||
			dragStartY === null ||
			dragStartImgX === null ||
			dragStartImgY === null ||
			editMode !== EditMode.Positioning
		) {
			return; // Only allow dragging in positioning mode
		}

		const dx = event.clientX - dragStartX;
		const dy = event.clientY - dragStartY;

		activeImage.transform.posX = dragStartImgX + dx;
		activeImage.transform.posY = dragStartImgY + dy;
	}

	function handleCanvasKey(event: KeyboardEvent) {
		if (!activeImage || editMode !== EditMode.Positioning) {
			return; // Only allow keyboard input in positioning mode
		}

		const step = 10; // Pixels to move per key press
		switch (event.key) {
			case 'ArrowUp':
				activeImage.transform.posY -= step;
				break;
			case 'ArrowDown':
				activeImage.transform.posY += step;
				break;
			case 'ArrowLeft':
				activeImage.transform.posX -= step;
				break;
			case 'ArrowRight':
				activeImage.transform.posX += step;
				break;
			case 'PageUp':
				zoomIn();
				break;
			case 'PageDown':
				zoomOut();
				break;
		}
	}

	function handleCanvasScroll(event: WheelEvent) {
		if (!activeImage || editMode !== EditMode.Positioning) {
			return; // Only allow zooming in positioning mode
		}

		event.preventDefault(); // Prevent default scrolling behavior

		if (event.deltaY < 0) {
			zoomIn();
		} else {
			zoomOut();
		}
	}

	function zoomIn() {
		if (!activeImage) return;

		activeImage.transform.scale += activeImage.transform.scale * 0.1;
		if (activeImage.transform.scale > 10) {
			activeImage.transform.scale = 10; // Limit max scale
		}
	}

	function zoomOut() {
		if (!activeImage) return;

		activeImage.transform.scale -= activeImage.transform.scale * 0.1;
		if (activeImage.transform.scale < 0.1) {
			activeImage.transform.scale = 0.1; // Limit min scale
		}
	}
</script>

<div class="flex h-screen max-h-screen flex-row items-stretch justify-center p-4">
	<Card.Root class="max-w-fit min-w-fit">
		<Card.Header>
			<Card.Title>Image Selection</Card.Title>
		</Card.Header>
		<Card.Content class="flex min-h-0 flex-1 flex-col gap-2">
			<Input
				bind:ref={fileInput}
				class="bg-primary! text-primary-foreground hover:bg-primary/90! focus-visible:border-ring focus-visible:ring-ring/50 rounded-md border-none text-sm font-medium transition-all outline-none"
				id="picture"
				type="file"
				accept="image/*"
				multiple
				onchange={() => loadImages()}
			/>

			{#if loadedImages.length > 0}
				<div class="min-h-0 flex-1 overflow-auto">
					{#if loadedImages.length > 0}
						{#each loadedImages as image, index}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div
								class={'hover:border-primary mb-2 flex cursor-pointer flex-col items-center justify-center rounded border-2 p-2 ' +
									(activeImage?.name === image.name ? 'border-primary' : '')}
								onclick={() => setActiveImage(image)}
							>
								<img
									class="center"
									src={URL.createObjectURL(new Blob([image.data]))}
									alt={image.name}
									style="max-width: 200px; max-height: 200px;"
								/>
								<span class="text-muted-foreground">{image.name}</span>
							</div>
						{/each}
					{:else}
						<p class="text-muted-foreground">No images loaded.</p>
					{/if}
				</div>
			{/if}
		</Card.Content>
	</Card.Root>
	<div class="flex flex-3/5 items-center justify-center p-4">
		<div class="relative aspect-square w-full max-w-[80vh]">
			<div class="visible absolute -top-4 left-1/2 -translate-x-1/2 -translate-y-full transform">
				<Button
					onclick={() => (editMode = EditMode.Positioning)}
					class={editMode === EditMode.Positioning ? '' : 'bg-secondary'}><Scaling /></Button
				>
				<Button
					onclick={() => (editMode = EditMode.Painting)}
					class={editMode === EditMode.Painting ? '' : 'bg-secondary'}><Paintbrush /></Button
				>
			</div>
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="outline-secondary focus:outline-primary absolute top-0 right-0 bottom-0 left-0 overflow-hidden outline-2 select-none"
				bind:clientWidth={canvasSize}
				onclick={e => handleCanvasClick(e)}
				oncontextmenu={e => {
					e.preventDefault();
					handleCanvasClick(e);
				}}
				ondragstart={e => handleCanvasDragStart(e)}
				ondragover={e => handleCanvasDrag(e)}
				tabindex="0"
				onkeydown={e => handleCanvasKey(e)}
				onwheel={e => handleCanvasScroll(e)}
			>
				{#if activeImage}
					<img
						src={URL.createObjectURL(new Blob([activeImage.data]))}
						alt={activeImage.name}
						class="absolute top-1/2 left-1/2"
						style="transform: translate(calc({(activeImage.transform.posX ?? 0) *
							canvasSizeMul}px - 50%), calc({(activeImage.transform.posY ?? 0) *
							canvasSizeMul}px - 50%)) scale({(activeImage.transform.scale ?? 1.0) *
							canvasSizeMul}); max-width: unset; max-height: unset;"
					/>
				{/if}
				<svg class="absolute" height={canvasSize} width={canvasSize}>
					<circle
						r={dimensions.stencilRadius * canvasSizeMul}
						cx={canvasSize / 2}
						cy={canvasSize / 2}
						fill="#ff000030"
					></circle>
				</svg>

				{#if activeImage}
					<img bind:this={maskImg} alt={''} class="absolute top-0 right-0 bottom-0 left-0 h-full w-full" />
				{/if}
			</div>
		</div>
	</div>
	<Card.Root class="min-w-[400px] flex-1/5">
		<Card.Header>
			<Card.Title>Editor Controls</Card.Title>
		</Card.Header>
		<Card.Content>
			{#if activeImage}
				<div class="grid grid-cols-2 gap-4">
					<Label>Token size</Label>
					<Select.Root type="single" bind:value={activeImage.size} onValueChange={() => onDimsChange()}>
						<Select.Trigger class="w-full">{activeImage.size}</Select.Trigger>
						<Select.Content>
							{#each Object.values(CreatureSize) as size}
								<Select.Item value={size}>{size}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>

					<Label>Oversized token</Label>
					<Checkbox bind:checked={activeImage.oversized} onCheckedChange={() => onDimsChange()} />

					<!-- Scale -->
					<Label>Scale</Label>
					<div>
						<Button onclick={() => zoomOut()}>-</Button>
						{Math.round(activeImage.transform.scale * 100)}%
						<Button onclick={() => zoomIn()}>+</Button>
					</div>

					<!-- Positioning -->
					<Label>Position X</Label>
					<Input type="number" min="-1000" max="1000" bind:value={activeImage.transform.posX} />
					<Label>Position Y</Label>
					<Input type="number" min="-1000" max="1000" bind:value={activeImage.transform.posY} />

					<!-- Painting -->
					<Label>Paint size</Label>
					<Slider type="single" min={10} max={dimensions.size / 2} bind:value={brushSize}></Slider>

					<Dialog.Root
						onOpenChange={async open => {
							await tick();
							if (open && activeImage && previewImg) {
								console.log(`Rendering preview for ${activeImage.name}`);

								imageProcessor.render(
									activeImage.data,
									$state.snapshot(dimensions),
									$state.snapshot(activeImage.transform),
									true,
									previewImg,
								);
							}
						}}
					>
						<Dialog.Trigger>Preview</Dialog.Trigger>
						<Dialog.Content class="w-3xl max-w-3xl sm:max-w-3xl">
							<Dialog.Header>
								<Dialog.Title>Preview</Dialog.Title>
								<Dialog.Description>
									A preview of your token with a simple white border and dark background.
								</Dialog.Description>
							</Dialog.Header>
							<img bind:this={previewImg} alt="Preview" class="border border-gray-300" />
						</Dialog.Content>
					</Dialog.Root>
				</div>
			{:else}
				<p>No image selected for editing.</p>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
