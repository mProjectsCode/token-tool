<script lang="ts">
	import {
		CreatureSize,
		EditMode,
		getImageDimensions,
		readFileAsArrayBuffer,
		remapRange,
		Throttle,
	} from '../../helpers/utils';
	import * as Card from '$lib/components/ui/card/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { ImageProcessor } from '../../helpers/image/imageProcessor';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import { tick, untrack } from 'svelte';
	import { Slider } from '$lib/components/ui/slider/index.js';
	import type { ImageTransform } from 'src/helpers/image/imageWorkerRPC';
	import { Paintbrush, Scaling } from '@lucide/svelte';
	import * as Select from '$lib/components/ui/select/index.js';
	import { Checkbox } from '$lib/components/ui/checkbox/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import type { Paintable } from 'src/helpers/paintable/paintable';
	import PaintableCanvas from './PaintableCanvas.svelte';
	import { PreviewImageHolder } from '../../helpers/image/previewImageHolder.svelte';

	interface LoadedImage {
		name: string;
		data: Uint8Array;
		mask: ImageData | undefined;
		size: CreatureSize;
		oversized: boolean;
		transform: ImageTransform;
	}

	interface DragState {
		startX: number;
		startY: number;
		startImgX: number;
		startImgY: number;
		leftMouseButton: boolean;
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
	let activeImage = $derived<LoadedImage | undefined>(
		activeImageIndex !== undefined ? loadedImages[activeImageIndex] : undefined,
	);
	let dimensions = $derived(
		activeImage
			? getImageDimensions(activeImage.size, activeImage.oversized)
			: getImageDimensions(CreatureSize.Medium, false),
	);
	let brushSize = $state<number>(40);
	let editMode = $state<EditMode>(EditMode.Positioning);

	let previewImgHolder = $state<PreviewImageHolder>(new PreviewImageHolder());
	let canvasWrapper = $state<HTMLDivElement | undefined>(undefined);
	let fileInput = $state<HTMLInputElement | null>();

	let paintable = $state<Paintable | undefined>(undefined);
	let canvasSize = $state<number>(0);
	let canvasSizeMul = $derived<number>(canvasSize > 0 ? canvasSize / dimensions.size : 1);

	async function setActiveImage(index: number) {
		saveMask();

		activeImageIndex = index;
		await tick(); // Ensure the activeImage is updated before proceeding
		if (paintable && activeImage) {
			paintable.loadImageData(activeImage.mask, dimensions);
		}
	}

	function handleCanvasClick(event: MouseEvent) {
		if (!activeImage) {
			return; // No active image to edit
		}

		if (editMode === EditMode.Painting) {
			// updateMask(event.clientX, event.clientY, event.button === 0);
		}
	}

	function updateMask(posX: number, posY: number, add: boolean) {
		if (!activeImage || !canvasWrapper || editMode !== EditMode.Painting) {
			return;
		}

		let x = remapRange(posX, 0, canvasSize, 0, dimensions.size);
		let y = remapRange(posY, 0, canvasSize, 0, dimensions.size);

		if (x < 0 || y < 0 || x > dimensions.size || y > dimensions.size) {
			return; // Click outside the image bounds
		}

		if (paintable) {
			paintable.drawCircle(x, y, brushSize, add);
		}
	}

	let dragState = $state<DragState | null>(null);
	let dragBrushThrottle = new Throttle((posX: number, posY: number, add: boolean) => {
		if (!activeImage || editMode !== EditMode.Painting) {
			return; // Only allow drag painting in painting mode
		}

		updateMask(posX, posY, add);
	}, 20);

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
		if (!activeImage) {
			return;
		}

		event.preventDefault(); // Prevent default scrolling behavior

		if (editMode === EditMode.Painting) {
			// In painting mode, use scroll to change brush size
			if (event.deltaY < 0) {
				brushSize += 5;
			} else {
				brushSize = Math.max(10, brushSize - 5); // Decrease brush size, but not below 10
			}
			brushSize = Math.min(dimensions.size / 2, brushSize); // Limit max brush size
			return;
		} else if (editMode === EditMode.Positioning) {
			if (event.deltaY < 0) {
				zoomIn();
			} else {
				zoomOut();
			}
		}
	}

	let mouseInCanvas = $state<boolean>(false);
	let mouseCanvasX = $state<number>(0);
	let mouseCanvasY = $state<number>(0);

	function handleCanvasMouseDown(event: MouseEvent) {
		if (!activeImage) return;

		dragState = {
			startX: mouseCanvasX,
			startY: mouseCanvasY,
			startImgX: activeImage.transform.posX,
			startImgY: activeImage.transform.posY,
			leftMouseButton: event.button === 0,
		};
	}

	function handleCanvasMouseUp(event: MouseEvent) {
		if (!activeImage) return;

		if (editMode === EditMode.Painting) {
			dragBrushThrottle.call(mouseCanvasX, mouseCanvasY, event.button === 0);
		}

		dragState = null;
	}

	function handleCanvasMouseOver(event: MouseEvent) {
		mouseInCanvas = true;
	}

	function handleCanvasMouseOut(event: MouseEvent) {
		dragState = null;
		mouseInCanvas = false;
	}

	function handleCanvasMouseMove(event: MouseEvent) {
		if (!mouseInCanvas || !activeImage || !canvasWrapper) return;

		const rect = canvasWrapper.getBoundingClientRect();
		mouseCanvasX = event.clientX - rect.left;
		mouseCanvasY = event.clientY - rect.top;

		if (dragState) {
			if (editMode === EditMode.Painting) {
				dragBrushThrottle.call(mouseCanvasX, mouseCanvasY, dragState.leftMouseButton);
			} else if (editMode === EditMode.Positioning) {
				const dx = mouseCanvasX - dragState.startX;
				const dy = mouseCanvasY - dragState.startY;

				activeImage.transform.posX = dragState.startImgX + Math.round(dx / canvasSizeMul);
				activeImage.transform.posY = dragState.startImgY + Math.round(dy / canvasSizeMul);
			}
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

	function saveMask() {
		if (!activeImage || !paintable) {
			return;
		}

		activeImage.mask = paintable.getImageData();
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
								onclick={() => setActiveImage(index)}
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
			<!-- svelte-ignore a11y_mouse_events_have_key_events -->
			<div
				class="outline-secondary focus:outline-primary absolute top-0 right-0 bottom-0 left-0 overflow-hidden outline-2"
				class:cursor-none={activeImage && editMode === EditMode.Painting}
				bind:clientWidth={canvasSize}
				bind:this={canvasWrapper}
				tabindex="0"
				onclick={e => handleCanvasClick(e)}
				oncontextmenu={e => {
					e.preventDefault();
					handleCanvasClick(e);
				}}
				onmousedown={e => handleCanvasMouseDown(e)}
				onmouseup={e => handleCanvasMouseUp(e)}
				onkeydown={e => handleCanvasKey(e)}
				onwheel={e => handleCanvasScroll(e)}
				onmouseenter={e => handleCanvasMouseOver(e)}
				onmouseleave={e => handleCanvasMouseOut(e)}
				onmousemove={e => handleCanvasMouseMove(e)}
				ondragstart={e => e.preventDefault()}
			>
				{#if activeImage}
					<img
						draggable="false"
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
					<!-- <img bind:this={maskImg} alt={''} class="absolute top-0 right-0 bottom-0 left-0 h-full w-full" /> -->
					<PaintableCanvas bind:paintable={paintable} bind:dimensions={dimensions}></PaintableCanvas>
				{/if}

				{#if activeImage && editMode === EditMode.Painting && mouseInCanvas}
					<svg class="absolute top-0 right-0 bottom-0 left-0 h-full w-full">
						<circle cx={mouseCanvasX} cy={mouseCanvasY} r={(brushSize * canvasSizeMul) / 2} fill="#ff000030"
						></circle>
					</svg>
				{/if}
			</div>
		</div>
	</div>
	<Card.Root class="min-w-[400px] flex-1/5">
		<Card.Header>
			<Card.Title>Editor Controls</Card.Title>
		</Card.Header>
		<Card.Content class="flex h-full flex-col">
			{#if activeImage}
				<div class="flex-1">
					<div class="grid grid-cols-2 gap-4">
						<Label>Token size</Label>
						<Select.Root type="single" bind:value={activeImage.size}>
							<Select.Trigger class="w-full">{activeImage.size}</Select.Trigger>
							<Select.Content>
								{#each Object.values(CreatureSize) as size}
									<Select.Item value={size}>{size}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>

						<Label>Oversized token</Label>
						<Checkbox bind:checked={activeImage.oversized} />

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
					</div>
				</div>
				<div class="mt-4 grid grid-cols-2 gap-4">
					<Dialog.Root
						onOpenChange={async open => {
							await tick();
							if (open && activeImage) {
								console.log(`Rendering preview for ${activeImage.name}`);

								saveMask();

								previewImgHolder.clearImage();
								imageProcessor.render(
									activeImage.data,
									activeImage.mask ? new Uint8Array(activeImage.mask.data) : undefined,
									$state.snapshot(dimensions),
									$state.snapshot(activeImage.transform),
									true,
									previewImgHolder,
								);
							}
						}}
					>
						<Dialog.Trigger>
							<Button variant="outline" class="w-full">Preview</Button>
						</Dialog.Trigger>
						<Dialog.Content class="w-3xl max-w-3xl sm:max-w-3xl">
							<Dialog.Header>
								<Dialog.Title>Preview</Dialog.Title>
								<Dialog.Description>
									A preview of your token with a simple white border and dark background.
								</Dialog.Description>
							</Dialog.Header>
							<div class="max-h-[80vh] overflow-auto">
								{#if previewImgHolder.image}
									<img
										src={previewImgHolder.image}
										alt="Preview"
										class="aspect-square w-full border"
									/>
									<img
										src={previewImgHolder.image}
										alt="Preview"
										class="aspect-square w-full border bg-white"
									/>
								{:else}
									<span class="text-muted-foreground">Rendering preview...</span>
								{/if}
							</div>
						</Dialog.Content>
					</Dialog.Root>

					{#if (activeImageIndex ?? 0) < loadedImages.length - 1}
						<Button
							variant="outline"
							class="w-full"
							onclick={() => {
								if (activeImageIndex !== undefined && activeImageIndex < loadedImages.length - 1) {
									setActiveImage(activeImageIndex + 1);
								} else {
									console.warn('No next image to edit');
								}
							}}
						>
							Next
						</Button>
					{:else}
						<Button
							variant="outline"
							class="w-full"
							onclick={() => {
								console.warn('Export not implemented yet');
							}}
						>
							Export
						</Button>
					{/if}
				</div>
			{:else}
				<p>No image selected for editing.</p>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
