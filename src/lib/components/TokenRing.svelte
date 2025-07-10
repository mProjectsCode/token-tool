<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import type { ImageProcessor } from 'src/helpers/image/imageProcessor';
	import { Input } from '$lib/components/ui/input/index.js';
	import Button from './ui/button/button.svelte';
	import { readFileAsArrayBuffer } from 'src/helpers/utils';

	let {
		open = $bindable(),
		imageProcessor,
	}: {
		open: boolean;
		imageProcessor: ImageProcessor;
	} = $props();

	let imgInput = $state<HTMLInputElement | null>();
	let jsonInput = $state<HTMLInputElement | null>();

	let status = $state<'idle' | 'loading' | 'loaded' | 'error'>('idle');
	let errorMessage = $state<string | null>(null);

	async function loadTokenRing() {
		if (!imgInput || !jsonInput) {
			console.error('Missing input elements.');
			return;
		}

		const imgFile = imgInput.files?.[0];
		const jsonFile = jsonInput.files?.[0];

		if (!imgFile || !jsonFile) {
			console.error('Please select both an image and a JSON file.');
			return;
		}

		status = 'loading';

		try {
			const imgBuffer = await readFileAsArrayBuffer(imgFile);
			const imgData = new Uint8Array(imgBuffer);
			const jsonData = await jsonFile.text();

			console.log(`loaded files, loading border...`);

			await imageProcessor.loadBorder(imgData, jsonData);

			status = 'loaded';
		} catch (error) {
			status = 'error';
			errorMessage = error instanceof Error ? error.message : 'Unknown error';

			console.error('Error loading token ring:', error);
		}
	}

	function reset() {
		status = 'idle';
		errorMessage = null;
		if (imgInput) imgInput.value = '';
		if (jsonInput) jsonInput.value = '';
	}
</script>

<Dialog.Root bind:open={open} onOpenChange={() => reset()}>
	<Dialog.Content class="w-[80vw]! max-w-[80vw]!">
		<Dialog.Header>
			<Dialog.Title>Token Ring</Dialog.Title>
			<Dialog.Description>Select a token ring from your file system.</Dialog.Description>
		</Dialog.Header>

		<Input
			bind:ref={imgInput}
			id="token-img"
			type="file"
			accept="image/jpeg,image/png,image/webp"
			disabled={status === 'loading'}
			onchange={() => (status = 'idle')}
		/>

		<Input
			bind:ref={jsonInput}
			id="token-json"
			type="file"
			accept=".json"
			disabled={status === 'loading'}
			onchange={() => (status = 'idle')}
		/>

		<Button onclick={() => loadTokenRing()} disabled={status !== 'idle'}>Load token ring</Button>

		{#if status === 'loading'}
			<p>Loading...</p>
		{:else if status === 'loaded'}
			<p>Token ring loaded successfully!</p>
		{:else if status === 'error'}
			<p class="text-red-500">Error: {errorMessage}</p>
		{/if}
	</Dialog.Content>
</Dialog.Root>
