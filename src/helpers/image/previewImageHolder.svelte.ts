export class PreviewImageHolder {
	image: string | null = $state(null);
	error: string | null = $state(null);

	constructor() {}

	setImage(img: Blob): void {
		if (this.image) {
			URL.revokeObjectURL(this.image);
		}
		this.image = URL.createObjectURL(img);
	}

	setError(err: string | null): void {
		this.error = err;
	}

	setImageFromData(data: Uint8Array): void {
		if (this.image) {
			URL.revokeObjectURL(this.image);
		}

		const blob = new Blob([data as BlobPart], { type: 'image/webp' });
		this.image = URL.createObjectURL(blob);
	}

	clearImage(): void {
		if (this.image) {
			URL.revokeObjectURL(this.image);
			this.image = null;
		}
	}
}
