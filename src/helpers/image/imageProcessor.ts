import type {
	ImageDimensions,
	ImageTransform,
	ImageWorkerRPCHandlersMain,
	ImageWorkerRPCHandlersWorker,
} from './imageWorkerRPC';
import { RPCController } from '../RPC';
import ImageWorker from './imageWorker?worker';
import type { PreviewImageHolder } from './previewImageHolder.svelte';

interface RenderTask {
	type: 'render';
	data: Uint8Array;
	mask: Uint8Array | undefined;
	dims: ImageDimensions;
	state: ImageTransform;
	ring: boolean;
	img: PreviewImageHolder;
}

type Task = RenderTask;

export class ImageProcessor {
	worker: Worker;
	RPC: RPCController<ImageWorkerRPCHandlersMain, ImageWorkerRPCHandlersWorker>;

	initialized: boolean = false;
	queue: Task[] = [];
	currentTask: Task | null = null;

	constructor() {
		this.worker = new ImageWorker();

		this.RPC = RPCController.toWorker<ImageWorkerRPCHandlersMain, ImageWorkerRPCHandlersWorker>(this.worker, {
			onInitialized: () => {
				console.log('Worker initialized');
				this.initialized = true;
			},
			onRenderFinished: img => {
				console.log('Render finished');
				if (!this.currentTask || this.currentTask.type !== 'render') {
					console.error('No current task to finish or task type mismatch');
					return;
				}

				this.currentTask.img.setImage(new Blob([img], { type: 'image/webp' }));
				this.currentTask = null;
				this.update();
			},
			log: (message: string) => {
				console.log('Worker log:', message);
			},
		});
		this.RPC.call('initialize', undefined);
	}

	private update() {
		if (this.queue.length === 0 || !this.initialized || this.currentTask) {
			return;
		}

		const task = this.queue.shift();
		if (!task) {
			return;
		}
		this.currentTask = task;
		if (task.type === 'render') {
			this.RPC.call('render', undefined, task.data, task.mask, task.dims, task.state, task.ring);
			return;
		}
	}

	render(
		data: Uint8Array,
		mask: Uint8Array | undefined,
		dims: ImageDimensions,
		state: ImageTransform,
		ring: boolean,
		img: PreviewImageHolder,
	): void {
		if (!this.initialized) {
			console.warn('Worker not initialized yet');
			return;
		}

		this.queue.push({ type: 'render', data, mask, dims, state, ring, img });

		this.update();
	}
}
