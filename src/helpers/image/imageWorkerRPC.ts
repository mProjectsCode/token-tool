/* eslint-disable */
// eslint turns them into interfaces which causes TS errors

import type { ImageRenderOptions } from 'image-processing/pkg/image_processing';

export type ImageWorkerRPCHandlersWorker = {
	initialize: [];
	render: [Uint8Array, Uint8Array | undefined, ImageRenderOptions];
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
