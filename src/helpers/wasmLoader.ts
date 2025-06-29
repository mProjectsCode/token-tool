import init from '../../image-processing/pkg/image_processing';
import wasmbin from '../../image-processing/pkg/image_processing_bg.wasm?url';

export async function loadWasm() {
	await init({ module_or_path: wasmbin });
}

export * as wasm from '../../image-processing/pkg/image_processing';
