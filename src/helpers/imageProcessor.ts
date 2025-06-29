import type {
	ImageDimensions,
	ImageTransform,
	ImageWorkerRPCHandlersMain,
	ImageWorkerRPCHandlersWorker,
} from './imageWorkerRPCConfig';
import { RPCController } from './RPC';
import ImageWorker from './imageWorker?worker';

interface RenderTask {
	type: 'render';
	data: Uint8Array;
	dims: ImageDimensions;
	state: ImageTransform;
	ring: boolean;
	img: HTMLImageElement;
}

interface UpdateMaskTask {
	type: 'updateMask';
	size: number;
	add: boolean;
	x: number;
	y: number;
	cb: (img: Uint8Array) => void;
}

interface SetMaskTask {
	type: 'setMask';
	data: Uint8Array;
	dims: ImageDimensions;
}

interface ClearMaskTask {
	type: 'clearMask';
	dims: ImageDimensions;
	cb: (img: Uint8Array) => void;
}

type Task = RenderTask | UpdateMaskTask | SetMaskTask | ClearMaskTask;

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

				this.currentTask.img.src = URL.createObjectURL(new Blob([img], { type: 'image/webp' }));
				this.currentTask = null;
				this.update();
			},
			onMaskUpdated: img => {
				console.log('Mask updated');
				if (
					!this.currentTask ||
					(this.currentTask.type !== 'updateMask' &&
						this.currentTask.type !== 'setMask' &&
						this.currentTask.type !== 'clearMask')
				) {
					console.error('No current task to finish or task type mismatch');
					return;
				}
				if (img && this.currentTask.type !== 'setMask') {
					this.currentTask.cb(img);
				}
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
		if (task.type === 'clearMask') {
			this.RPC.call('clearMask', undefined, task.dims);
			return;
		}
		if (task.type === 'setMask') {
			this.RPC.call('setMask', undefined, task.data, task.dims);
			return;
		}
		if (task.type === 'updateMask') {
			this.RPC.call('drawOnMask', undefined, task.size, task.add, task.x, task.y);
			return;
		}
		if (task.type === 'render') {
			this.RPC.call('render', undefined, task.data, task.dims, task.state, task.ring);
			return;
		}
	}

	render(data: Uint8Array, dims: ImageDimensions, state: ImageTransform, ring: boolean, img: HTMLImageElement): void {
		if (!this.initialized) {
			console.warn('Worker not initialized yet');
			return;
		}

		this.queue.push({ type: 'render', data, dims, state, ring, img });

		this.update();
	}

	updateMask(size: number, add: boolean, x: number, y: number, cb: (img: Uint8Array) => void): void {
		if (!this.initialized) {
			console.warn('Worker not initialized yet');
			return;
		}

		this.queue.push({ type: 'updateMask', size, add, x, y, cb });

		this.update();
	}

	setMask(data: Uint8Array, dims: ImageDimensions): void {
		if (!this.initialized) {
			console.warn('Worker not initialized yet');
			return;
		}

		this.queue.push({ type: 'setMask', data, dims });

		this.update();
	}

	clearMask(dims: ImageDimensions, cb: (img: Uint8Array) => void): void {
		if (!this.initialized) {
			console.warn('Worker not initialized yet');
			return;
		}

		this.queue.push({ type: 'clearMask', dims, cb });

		this.update();
	}
}
