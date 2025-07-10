/* eslint-disable */
// eslint turns them into interfaces which causes TS errors

export interface ImageTransform {
	scale: number;
	posX: number;
	posY: number;
	flipped: boolean;
}

export interface ImageDimensions {
	size: number;
	oversized: boolean;
	stencilRadius: number;
}

export type ImageWorkerRPCHandlersWorker = {
	initialize: [];
	render: [Uint8Array, Uint8Array | undefined, ImageDimensions, ImageTransform, boolean];
	loadBorder: [Uint8Array, string];
	previewBorder: [];
};

export type ImageWorkerRPCHandlersMain = {
	onRenderFinished: [Uint8Array];
	onLoadBorderFinished: [];
	onPreviewBorderFinished: [Uint8Array];
	onError: [string];
	onInitialized: [];
	log: [string];
};
