import type { ImageTransform, ImageDimensions, ImageWorkerRPCHandlersMain, ImageWorkerRPCHandlersWorker } from './imageWorkerRPCConfig';
import { RPCController } from './RPC';
import init, { ImageProcessor, ImageTransform as IT, ImageDimensions as ID } from '../../image-processing/pkg/image_processing';
import wasmbin from '../../image-processing/pkg/image_processing_bg.wasm?url';

let imageProcessor: ImageProcessor | null = null;

function convertDimensions(dimensions: ImageDimensions): ID {
    return new ID(dimensions.size, dimensions.stencilRadius);
}

const RPC = new RPCController<ImageWorkerRPCHandlersWorker, ImageWorkerRPCHandlersMain>(
	{
		initialize() {
			init({ module_or_path: wasmbin }).then(() => {
				imageProcessor = new ImageProcessor();
				RPC.call('onInitialized', undefined);
			});
		},
		render(data: Uint8Array, dimensions: ImageDimensions, state: ImageTransform, ring: boolean) {
			const transform = new IT(state.posX, state.posY, state.scale, state.flipped);
            const dims = convertDimensions(dimensions);

			const img = imageProcessor!.render(data, dims, transform, ring);

			RPC.call('onRenderFinished', undefined, img);
		},
		clearMask(dimensions: ImageDimensions) {
            const dims = convertDimensions(dimensions);
			const mask = imageProcessor!.clear_mask(dims);

			RPC.call('onMaskUpdated', undefined, mask);
		},
		setMask(data: Uint8Array, dimensions: ImageDimensions) {
            const dims = convertDimensions(dimensions);
			imageProcessor!.set_mask(data, dims);

			RPC.call('onMaskUpdated', undefined, undefined);
		},
		drawOnMask(size: number, add: boolean, x: number, y: number) {
			const mask = imageProcessor!.draw_on_mask(size, add, x, y);
			RPC.call('onMaskUpdated', undefined, mask);
		},
	},
	m => postMessage(m),
);

onmessage = (e: MessageEvent) => {
	RPC.handle(e.data);
};
