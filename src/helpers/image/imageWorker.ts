import type {
	ImageTransform,
	ImageDimensions,
	ImageWorkerRPCHandlersMain,
	ImageWorkerRPCHandlersWorker,
} from './imageWorkerRPC';
import { RPCController } from '../RPC';
import init, {
	ImageProcessor,
	ImageTransform as IT,
	ImageDimensions as ID,
} from '../../../image-processing/pkg/image_processing';
import wasmbin from '../../../image-processing/pkg/image_processing_bg.wasm?url';

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
		render(
			data: Uint8Array,
			mask: Uint8Array | undefined,
			dimensions: ImageDimensions,
			state: ImageTransform,
			ring: boolean,
		) {
			const transform = new IT(state.posX, state.posY, state.scale, state.flipped);
			const dims = convertDimensions(dimensions);

			try {
				const img = imageProcessor!.render(data, mask, dims, transform, ring);
				RPC.call('onRenderFinished', undefined, img);
			} catch (error) {
				RPC.call('onRenderError', undefined, error instanceof Error ? error.message : String(error));
			}
		},
	},
	m => postMessage(m),
);

onmessage = (e: MessageEvent) => {
	RPC.handle(e.data);
};
