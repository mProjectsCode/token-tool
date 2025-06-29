import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import path from 'path';

// https://vite.dev/config/
export default defineConfig({
	plugins: [tailwindcss(), svelte(), wasm(), topLevelAwait()],
	resolve: {
		alias: {
			$lib: path.resolve('./src/lib'),
		},
	},
});
