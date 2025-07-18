export type RPCConfig = Record<string, unknown[]>;

export type RPCMethods<TConfig extends RPCConfig> = {
	[K in keyof TConfig]: (...args: TConfig[K]) => void;
};

/**
 * Controller for a remote procedure call (RPC) system.
 *
 * @template THandlers - The configuration of the methods that this side handles.
 * @template TMethods - The configuration of the methods that can be called.
 */
export class RPCController<THandlers extends RPCConfig, TMethods extends RPCConfig> {
	private handlers: RPCMethods<THandlers>;
	private post: (message: unknown, transferable?: Transferable[]) => void;

	/**
	 * Creates a new RPC controller.
	 *
	 * @param handlers - The handlers for the methods that this side handles.
	 * @param post - The function to call to send a message to the other side.
	 */
	constructor(handlers: RPCMethods<THandlers>, post: (message: unknown, transferable?: Transferable[]) => void) {
		this.handlers = handlers;
		this.post = post;
	}

	static toWorker<THandlers extends RPCConfig, TMethods extends RPCConfig>(
		worker: Worker,
		handlers: RPCMethods<THandlers>,
	): RPCController<THandlers, TMethods> {
		const controller = new RPCController(handlers, (message, transferable) => {
			if (transferable) {
				worker.postMessage(message, transferable);
			} else {
				worker.postMessage(message);
			}
		});
		worker.onmessage = (e: MessageEvent): void => controller.handle(e.data);
		return controller;
	}

	/**
	 * Calls a method on the other side.
	 *
	 * @param method - The method to call.
	 * @param args - The arguments to pass to the method.
	 */
	call<K extends keyof TMethods>(method: K, transferable: Transferable[] | undefined, ...args: TMethods[K]): void {
		// console.log('calling', method, args);

		this.post({ method, args }, transferable);
	}

	/**
	 * Handles a call from the other side.
	 * Will do nothing if the call is malformed.
	 *
	 * @param call - The call to handle.
	 */
	handle(call: unknown): void {
		// console.log('received', message);

		if (typeof call !== 'object' || call === null) {
			return;
		}

		const { method, args } = call as { method: keyof THandlers; args: unknown[] };
		if (typeof method !== 'string') {
			return;
		}

		const handler = this.handlers[method];
		if (handler) {
			// @ts-expect-error
			handler(...args);
		}
	}
}
