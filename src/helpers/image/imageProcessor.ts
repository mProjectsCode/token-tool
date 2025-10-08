import type { ImageWorkerRPCHandlersMain, ImageWorkerRPCHandlersWorker } from './imageWorkerRPC';
import { RPCController } from '../RPC';
import ImageWorker from './imageWorker?worker';
import { assertType } from '../utils';
import type { ImageRenderOptions } from 'image-processing/pkg/image_processing';

interface RenderTask {
	type: 'render';
	data: Uint8Array;
	mask: Uint8Array | undefined;
	opts: ImageRenderOptions;
	cb: (img: Uint8Array | string) => void;
}

interface LoadBorderTask {
	type: 'loadBorder';
	data: Uint8Array;
	meta: string;
	cb: (error: undefined | string) => void;
}

interface PreviewBorderTask {
	type: 'previewBorder';
	cb: (img: Uint8Array | string) => void;
}

type Task = RenderTask | LoadBorderTask | PreviewBorderTask;

export class ImageProcessor {
	worker: Worker;
	RPC: RPCController<ImageWorkerRPCHandlersMain, ImageWorkerRPCHandlersWorker>;

	initialized: boolean = false;
	queue: Task[] = [];
	currentTask: Task | null = null;
	onInitialized: () => void = () => {};

	constructor() {
		this.worker = new ImageWorker();

		this.RPC = RPCController.toWorker<ImageWorkerRPCHandlersMain, ImageWorkerRPCHandlersWorker>(this.worker, {
			onInitialized: () => {
				this.onInitialized();
			},
			onRenderFinished: img => {
				console.log('Render finished');
				if (!this.currentTask || this.currentTask.type !== 'render') {
					console.error('No current task to finish or task type mismatch');
					return;
				}

				this.currentTask.cb(img);
				this.currentTask = null;
				this.update();
			},
			onLoadBorderFinished: () => {
				console.log('Load border finished');
				if (!this.currentTask || this.currentTask.type !== 'loadBorder') {
					console.error('No current task to finish or task type mismatch');
					return;
				}

				this.currentTask.cb(undefined);
				this.currentTask = null;
				this.update();
			},
			onPreviewBorderFinished: img => {
				console.log('Preview border finished');
				if (!this.currentTask || this.currentTask.type !== 'previewBorder') {
					console.error('No current task to finish or task type mismatch');
					return;
				}

				this.currentTask.cb(img);
				this.currentTask = null;
				this.update();
			},
			onError: msg => {
				console.log('Worker error:', msg);
				if (!this.currentTask) {
					console.error('No current task to finish');
					return;
				}

				this.currentTask.cb(msg);
				this.currentTask = null;
				this.update();
			},
			log: (message: string) => {
				console.log('Worker log:', message);
			},
		});
	}

	private async initialize(): Promise<void> {
		if (this.initialized) {
			return;
		}
		return new Promise<void>((resolve, reject) => {
			this.onInitialized = () => {
				console.log('Worker initialized');
				this.initialized = true;
				resolve();
			};
			this.RPC.call('initialize', undefined);
		});
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
			this.RPC.call('render', undefined, task.data, task.mask, task.opts);
			return;
		}
		if (task.type === 'loadBorder') {
			this.RPC.call('loadBorder', undefined, task.data, task.meta);
			return;
		}
		if (task.type === 'previewBorder') {
			this.RPC.call('previewBorder', undefined);
			return;
		}
		assertType<never>(task);
	}

	async render(data: Uint8Array, mask: Uint8Array | undefined, opts: ImageRenderOptions): Promise<Uint8Array> {
		await this.initialize();

		return new Promise((resolve, reject) => {
			if (!this.initialized) {
				console.warn('Worker not initialized yet');
				reject(new Error('Worker not initialized'));
				return;
			}

			this.queue.push({
				type: 'render',
				data,
				mask,
				opts,
				cb: img => {
					if (typeof img === 'string') {
						reject(new Error(`Image rendering error: ${img}`));
					} else {
						resolve(img);
					}
				},
			});

			this.update();
		});
	}

	async loadBorder(data: Uint8Array, meta: string): Promise<void> {
		await this.initialize();

		return new Promise((resolve, reject) => {
			if (!this.initialized) {
				console.warn('Worker not initialized yet');
				reject(new Error('Worker not initialized'));
				return;
			}

			this.queue.push({
				type: 'loadBorder',
				data,
				meta,
				cb: error => {
					if (error) {
						reject(new Error(`Border loading error: ${error}`));
					} else {
						resolve();
					}
				},
			});

			this.update();
		});
	}

	async previewBorder(): Promise<Uint8Array> {
		await this.initialize();

		return new Promise((resolve, reject) => {
			if (!this.initialized) {
				console.warn('Worker not initialized yet');
				reject(new Error('Worker not initialized'));
				return;
			}

			this.queue.push({
				type: 'previewBorder',
				cb: img => {
					if (typeof img === 'string') {
						reject(new Error(`Border preview error: ${img}`));
					} else {
						resolve(img);
					}
				},
			});

			this.update();
		});
	}
}
