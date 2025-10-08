import type { ImageDimensions, ImageTransform } from 'image-processing/pkg/image_processing';

export async function readFileAsArrayBuffer(file: File): Promise<Uint8Array> {
	return new Promise((resolve, reject) => {
		const fileReader = new FileReader();
		fileReader.onload = () => {
			if (fileReader.result && fileReader.result instanceof ArrayBuffer) {
				resolve(new Uint8Array(fileReader.result));
			} else {
				reject(new Error('Failed to read file as ArrayBuffer.'));
			}
		};
		fileReader.onerror = () => reject(fileReader.error);
		fileReader.readAsArrayBuffer(file);
	});
}

export enum EditMode {
	Positioning = 'Positioning',
	Painting = 'Painting',
}

export enum CreatureSize {
	Tiny = 'Tiny',
	Small = 'Small',
	Medium = 'Medium',
	Large = 'Large',
	Huge = 'Huge',
	Gargantuan = 'Gargantuan',
}

export function getImageDimensions(creatureSize: CreatureSize, oversized: boolean): ImageDimensions {
	let canvasSize: number;
	if (creatureSize === CreatureSize.Tiny) {
		canvasSize = 256;
	} else if (creatureSize === CreatureSize.Small || creatureSize === CreatureSize.Medium) {
		canvasSize = 512;
	} else if (creatureSize === CreatureSize.Large || creatureSize === CreatureSize.Huge) {
		canvasSize = 1024;
	} else if (creatureSize === CreatureSize.Gargantuan) {
		canvasSize = 2048;
	} else {
		throw new Error(`Unknown creature size: ${creatureSize}`);
	}

	let stencilRadius = Math.round(canvasSize / 3);

	if (oversized) {
		canvasSize *= 2;
	}

	return {
		size: canvasSize,
		oversized: oversized,
		stencil_radius: stencilRadius,
	};
}

export function remapRange(value: number, oldMin: number, oldMax: number, newMin: number, newMax: number): number {
	if (oldMin === oldMax) {
		throw new Error('Old range cannot be zero.');
	}
	return ((value - oldMin) * (newMax - newMin)) / (oldMax - oldMin) + newMin;
}

export function debounce<F extends (...args: Args) => void, const Args extends any[]>(
	callback: F,
	wait: number,
): (...args: Args) => void {
	let timeoutId: number | undefined = undefined;
	return (...args) => {
		window.clearTimeout(timeoutId);
		timeoutId = window.setTimeout(() => {
			callback(...args);
		}, wait);
	};
}

export class Throttle<F extends (...args: any[]) => void> {
	private intervalId: number | undefined = undefined;
	private lastArgs: Parameters<F> | undefined = undefined;

	constructor(
		private callback: F,
		private wait: number,
	) {
		this.intervalId = window.setInterval(() => {
			if (this.lastArgs !== undefined) {
				this.callback(...this.lastArgs);
				this.lastArgs = undefined;
			}
		}, this.wait);
	}

	public destroy(): void {
		if (this.intervalId) {
			window.clearInterval(this.intervalId);
			this.intervalId = undefined;
		}
	}

	public call(...args: Parameters<F>): void {
		if (!this.intervalId) {
			throw new Error('Throttle has been destroyed and cannot be used anymore.');
		}

		this.lastArgs = args;
	}
}

export interface LoadedImage {
	name: string;
	data: Uint8Array;
	imgUrl: string;
	mask: ImageData | undefined;
	size: CreatureSize;
	oversized: boolean;
	transform: ImageTransform;
	completed: boolean;
}

export function assertType<T>(value: T): void {
	// no-op function to assert type at compile time
}

export function removeExtension(filename: string): string {
	const lastDotIndex = filename.lastIndexOf('.');
	if (lastDotIndex === -1) {
		return filename; // No extension found
	}
	return filename.slice(0, lastDotIndex);
}
