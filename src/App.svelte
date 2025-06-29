<script lang="ts">
	import { ModeWatcher } from 'mode-watcher';
	import './app.css';
	import ImageEditor from '$lib/components/ImageEditor.svelte';
</script>

<main>
	<ModeWatcher />
	<ImageEditor />
</main>
