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
    stencilRadius: number;
}

export type ImageWorkerRPCHandlersWorker = {
	initialize: [];
	render: [Uint8Array, ImageDimensions, ImageTransform, boolean];
	clearMask: [ImageDimensions];
	setMask: [Uint8Array, ImageDimensions];
	drawOnMask: [number, boolean, number, number];
};

export type ImageWorkerRPCHandlersMain = {
	onRenderFinished: [Uint8Array];
	onMaskUpdated: [Uint8Array | undefined];
	onInitialized: [];
	log: [string];
};
