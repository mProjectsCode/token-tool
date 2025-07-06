export class PreviewImageHolder {
	image: string | null = $state(null);

	constructor() {}

	setImage(img: Blob): void {
		if (this.image) {
			URL.revokeObjectURL(this.image);
		}
		this.image = URL.createObjectURL(img);
	}

	clearImage(): void {
		if (this.image) {
			URL.revokeObjectURL(this.image);
			this.image = null;
		}
	}
}
