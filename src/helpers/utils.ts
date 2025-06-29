import type { ImageDimensions } from './imageWorkerRPCConfig';

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
		stencilRadius: stencilRadius,
	};
}

export function remapRange(value: number, oldMin: number, oldMax: number, newMin: number, newMax: number): number {
	if (oldMin === oldMax) {
		throw new Error('Old range cannot be zero.');
	}
	return ((value - oldMin) * (newMax - newMin)) / (oldMax - oldMin) + newMin;
}
