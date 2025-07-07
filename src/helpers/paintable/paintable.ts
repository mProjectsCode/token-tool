import type { ImageDimensions } from '../image/imageWorkerRPC';

export class Paintable {
	private canvas: HTMLCanvasElement;
	private ctx: CanvasRenderingContext2D;
	private fillStyle: string;

	constructor(element: HTMLCanvasElement, dimensions: ImageDimensions, fillStyle: string = 'white') {
		this.canvas = element;
		this.fillStyle = fillStyle;
		const ctx = this.canvas.getContext('2d');
		if (!ctx) {
			throw new Error('Failed to get 2D context from canvas');
		}
		this.ctx = ctx;

		this.clear(dimensions);
	}

	public clear(dimensions: ImageDimensions): void {
		this.canvas.width = dimensions.size;
		this.canvas.height = dimensions.size;
		this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
	}

	public loadImageData(imageData: ImageData | undefined, dimensions: ImageDimensions): void {
		this.clear(dimensions);

		if (imageData && imageData.width === dimensions.size && imageData.height === dimensions.size) {
			this.ctx.putImageData(imageData, 0, 0);
		}
	}

	public getImageData(): ImageData {
		return this.ctx.getImageData(0, 0, this.canvas.width, this.canvas.height);
	}

	public drawCircle(x: number, y: number, diameter: number, add: boolean): void {
		this.ctx.beginPath();
		if (!add) {
			this.ctx.globalCompositeOperation = 'destination-out';
		} else {
			this.ctx.globalCompositeOperation = 'source-over';
		}
		this.ctx.arc(x, y, diameter / 2, 0, Math.PI * 2);
		this.ctx.fillStyle = this.fillStyle;
		this.ctx.fill();
		this.ctx.closePath();
	}
}
