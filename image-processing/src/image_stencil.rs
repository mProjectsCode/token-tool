use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

use crate::{image_options::ImageDimensions};

#[allow(dead_code)]
pub trait ImageStencil {
    fn to_stencil<'a>(&'a self, threshold: u8) -> StencilMask<'a>;
    fn to_inverted_stencil<'a>(&'a self, threshold: u8) -> StencilMask<'a>;

    fn stencil(&self, mask: &StencilMask) -> DynamicImage;

    fn stencil_and(&self, masks: &[&StencilMask]) -> DynamicImage;

    fn stencil_or(&self, masks: &[&StencilMask]) -> DynamicImage;
}

impl ImageStencil for DynamicImage {
    fn to_stencil<'a>(&'a self, threshold: u8) -> StencilMask<'a> {
        StencilMask::new(self, false, threshold)
    }

    fn to_inverted_stencil<'a>(&'a self, threshold: u8) -> StencilMask<'a> {
        StencilMask::new(self, true, threshold)
    }

    fn stencil(&self, mask: &StencilMask) -> DynamicImage {
        ImageBuffer::from_fn(self.width(), self.height(), |x, y| {
            let mask_pixel = mask.image.get_pixel(x, y);
            if (mask_pixel[3] > mask.threshold) ^ mask.invert {
                self.get_pixel(x, y)
            } else {
                Rgba([0, 0, 0, 0]) // Transparent pixel
            }
        })
        .into()
    }

    fn stencil_and(&self, masks: &[&StencilMask]) -> DynamicImage {
        ImageBuffer::from_fn(self.width(), self.height(), |x, y| {
            let show_pixel = masks
                .iter()
                .all(|mask| (mask.image.get_pixel(x, y)[3] > mask.threshold) ^ mask.invert);

            if show_pixel {
                self.get_pixel(x, y)
            } else {
                Rgba([0, 0, 0, 0]) // Transparent pixel
            }
        })
        .into()
    }

    fn stencil_or(&self, masks: &[&StencilMask]) -> DynamicImage {
        ImageBuffer::from_fn(self.width(), self.height(), |x, y| {
            let show_pixel = masks
                .iter()
                .any(|mask| (mask.image.get_pixel(x, y)[3] > mask.threshold) ^ mask.invert);

            if show_pixel {
                self.get_pixel(x, y)
            } else {
                Rgba([0, 0, 0, 0]) // Transparent pixel
            }
        })
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct StencilMask<'a> {
    pub image: &'a DynamicImage,
    pub invert: bool,
    pub threshold: u8,
}

impl<'a> StencilMask<'a> {
    pub fn new(image: &'a DynamicImage, invert: bool, threshold: u8) -> Self {
        StencilMask {
            image,
            invert,
            threshold,
        }
    }
}

pub fn overlay_images(dimensions: &ImageDimensions, images: &[&DynamicImage]) -> DynamicImage {
    ImageBuffer::from_fn(dimensions.size, dimensions.size, |x, y| {
        let mut final_pixel = (0.0, 0.0, 0.0, 0.0); // (r, g, b, a)

        for image in images {
            let pixel = image.get_pixel(x, y);
            let alpha = pixel[3] as f32 / 255.0;

            final_pixel.0 = final_pixel.0 * (1.0 - alpha) + pixel[0] as f32 * alpha;
            final_pixel.1 = final_pixel.1 * (1.0 - alpha) + pixel[1] as f32 * alpha;
            final_pixel.2 = final_pixel.2 * (1.0 - alpha) + pixel[2] as f32 * alpha;
            final_pixel.3 = final_pixel.3 + alpha * (1.0 - final_pixel.3);
        }

        Rgba([
            final_pixel.0.clamp(0.0, 255.0) as u8,
            final_pixel.1.clamp(0.0, 255.0) as u8,
            final_pixel.2.clamp(0.0, 255.0) as u8,
            (final_pixel.3 * 255.0).clamp(0.0, 255.0) as u8,
        ])
    }).into()
}
