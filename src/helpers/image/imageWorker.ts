import type { ImageWorkerRPCHandlersMain, ImageWorkerRPCHandlersWorker } from './imageWorkerRPC';
import { RPCController } from '../RPC';
import init, { ImageProcessor, type ImageRenderOptions } from '../../../image-processing/pkg/image_processing';
import wasmbin from '../../../image-processing/pkg/image_processing_bg.wasm?url';

let imageProcessor: ImageProcessor | null = null;

const RPC = new RPCController<ImageWorkerRPCHandlersWorker, ImageWorkerRPCHandlersMain>(
	{
		initialize() {
			init({ module_or_path: wasmbin }).then(() => {
				imageProcessor = new ImageProcessor();
				RPC.call('onInitialized', undefined);
			});
		},
		render(data: Uint8Array, mask: Uint8Array | undefined, opts: ImageRenderOptions) {
			try {
				const img = imageProcessor!.render(data, mask, opts);
				RPC.call('onRenderFinished', undefined, img);
			} catch (error) {
				RPC.call('onError', undefined, error instanceof Error ? error.message : String(error));
			}
		},
		loadBorder: (img: Uint8Array, meta: string) => {
			try {
				imageProcessor!.load_border(img, meta);
				RPC.call('onLoadBorderFinished', undefined);
			} catch (error) {
				RPC.call('onError', undefined, error instanceof Error ? error.message : String(error));
			}
		},
		previewBorder: () => {
			// try {
			// 	const img = imageProcessor!.(img, meta);
			// 	RPC.call('onPreviewBorderFinished', undefined);
			// } catch (error) {
			// 	RPC.call('onError', undefined, error instanceof Error ? error.message : String(error));
			// }
		},
	},
	m => postMessage(m),
);

onmessage = (e: MessageEvent) => {
	RPC.handle(e.data);
};
