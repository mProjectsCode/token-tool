mod image_border;
mod image_options;
mod image_shadow;
mod image_stencil;
mod utils;

use std::io::Cursor;

use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba, imageops};
use imageproc::rect::Rect;
use wasm_bindgen::prelude::*;

use crate::{
    image_border::ImageBorder,
    image_options::{ImageDimensions, ImageRenderOptions, ImageTransform},
    image_shadow::{ImageShadow, ShadowOptions},
    image_stencil::{overlay_images, ImageStencil},
    utils::set_panic_hook,
};

fn create_blank_image(dimensions: &ImageDimensions) -> DynamicImage {
    let blank_image = image::ImageBuffer::new(dimensions.size, dimensions.size);
    DynamicImage::ImageRgba8(blank_image)
}

fn image_to_bytes(image: &DynamicImage) -> Result<Vec<u8>, JsValue> {
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::WebP,
        )
        .map_err(|e| JsValue::from_str(&format!("Failed to write image: {e}")))?;
    Ok(bytes)
}

#[wasm_bindgen]
pub struct ImageProcessor {
    border: Option<ImageBorder>,
}

#[wasm_bindgen]
impl ImageProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        set_panic_hook();

        Ok(ImageProcessor { border: None })
    }

    pub fn render(
        &self,
        image_data: &[u8],
        mask_data: Option<Vec<u8>>,
        options: ImageRenderOptions,
    ) -> Result<Vec<u8>, JsValue> {
        // Load the image data into a DynamicImage
        let image: DynamicImage = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()
            .map_err(|e| JsValue::from_str(&format!("Failed to read image: {e}")))?
            .decode()
            .map_err(|e| JsValue::from_str(&format!("Failed to decode image: {e}")))?;

        let mask: DynamicImage = match mask_data {
            Some(x) => {
                ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(options.dimensions.size, options.dimensions.size, x)
                    .ok_or(JsValue::from_str(
                        "Failed to create mask image from provided data",
                    ))?
                    .into()
            }
            None => create_blank_image(&options.dimensions),
        };

        let image = self.cut_and_transform(
            image,
            &options.dimensions,
            &options.transform,
        );

        let composite_image = self.build_image(&image, &mask, &options)?;

        image_to_bytes(&composite_image)
    }

    pub fn load_border(&mut self, image_data: &[u8], meta: String) -> Result<(), JsValue> {
        self.border = Some(ImageBorder::from_js(image_data, meta)?);

        Ok(())
    }
}

impl ImageProcessor {
    /// Cut the image to fit into the given dimensions, centering it and applying the given image transform.
    pub fn cut_and_transform(&self, image: DynamicImage, dimensions: &ImageDimensions, image_transform: &ImageTransform) -> DynamicImage {
        // first flip if needed
        let image = if image_transform.flipped {
            image.fliph()
        } else {
            image
        };

        // second scale
        let scaled_width = (image.width() as f32 * image_transform.scale) as u32;
        let scaled_height = (image.height() as f32 * image_transform.scale) as u32;

        let image= image.resize(
            scaled_width,
            scaled_height,
            imageops::FilterType::CatmullRom,
        );

        // then calculate the offsets
        let x_offset = (dimensions.size as i32 - image.width() as i32) / 2 + image_transform.pos_x;
        let y_offset = (dimensions.size as i32 - image.height() as i32) / 2 + image_transform.pos_y;

        // finally, cut the image by overlaying it on a blank canvas of the right size
        let mut tmp_image = create_blank_image(dimensions);
        imageops::overlay(&mut tmp_image, &image, x_offset as i64, y_offset as i64);

        tmp_image
    }

    pub fn create_ring_image(
        &self,
        dimension: &ImageDimensions,
        ring_width: u32,
    ) -> (DynamicImage, DynamicImage) {
        let mut ring_image = create_blank_image(dimension);
        imageproc::drawing::draw_filled_circle_mut(
            &mut ring_image,
            dimension.center_tuple_i32(),
            (dimension.stencil_radius + ring_width) as i32,
            image::Rgba([220, 220, 220, 255]),
        );
        imageproc::drawing::draw_filled_circle_mut(
            &mut ring_image,
            dimension.center_tuple_i32(),
            dimension.stencil_radius as i32,
            image::Rgba([0, 0, 0, 0]),
        );

        (create_blank_image(dimension), ring_image)
    }

    /// Build the final composite image with shadows and all.
    /// Assumes that all the images are already resized to the correct
    /// dimensions.
    pub fn build_image(
        &self,
        image: &DynamicImage,
        mask: &DynamicImage,
        options: &ImageRenderOptions,
    ) -> Result<DynamicImage, JsValue> {
        let circle_mask = self.create_stencil(&options.dimensions);
        let circle_mask_inverted = self.create_inverted_stencil(&options.dimensions);
        // circle mask that only keeps the center circle
        let circle_stencil = circle_mask.to_stencil(0);
        // inverted circle mask that keeps everything outside the center circle
        // let circle_stencil_inverted = circle_mask.to_inverted_stencil();

        let mask_stencil = mask.to_stencil(0);
        let mask_stencil_inverted = mask.to_inverted_stencil(0);
        let masked_image = image.stencil(&mask_stencil);
        let masked_image_inverted = image.stencil_and(&[&mask_stencil_inverted, &circle_stencil]);

        let (ring_bg, ring_fg) = if options.ring {
            if let Some(border) = &self.border {
                border.get_ring(&options.dimensions)?
            } else {
                // If no border is loaded, create a default ring image
                self.create_ring_image(&options.dimensions, 20)
            }
        } else {
            (
                create_blank_image(&options.dimensions),
                create_blank_image(&options.dimensions),
            )
        };

        let image_shadow_options = ShadowOptions::new_black(0.4, 3.0, 5, 5);
        let image_shadow = image.to_shadow(&image_shadow_options);
        let image_shadow_mask = image_shadow.stencil(&mask_stencil);
        let image_shadow_non_mask =
            image_shadow.stencil_and(&[&mask_stencil_inverted, &circle_stencil]);

        let ring_shadow_options = ShadowOptions::new_black(0.8, 10.0, 7, 12);
        let ring_shadow = circle_mask_inverted.to_shadow(&ring_shadow_options);
        let stenciled_ring_shadow = ring_shadow.stencil(&circle_stencil);

        Ok(overlay_images(
            &options.dimensions,
            &[
                &ring_bg,               // the ring image background
                &ring_fg,               // the ring image foreground
                &image_shadow_non_mask, // the image shadow everywhere except the masked area
                &masked_image_inverted, // the image in the circle area except the masked area
                &stenciled_ring_shadow, // the ring shadow in the circle area
                &image_shadow_mask,     // the image shadow in the masked area
                &masked_image,          // the image in the masked area
            ],
        ))
    }

    pub fn mask_and_stencil_image(
        &self,
        image: &DynamicImage,
        mask: &DynamicImage,
        dimensions: &ImageDimensions,
    ) -> DynamicImage {
        ImageBuffer::from_fn(dimensions.size, dimensions.size, |x, y| {
            let distance_to_center = (x as i32 - (dimensions.size / 2) as i32).pow(2)
                + (y as i32 - (dimensions.size / 2) as i32).pow(2);
            let in_stencil_area = distance_to_center <= dimensions.stencil_radius.pow(2) as i32;
            let is_masked = mask.get_pixel(x, y).0[3] > 0;

            if in_stencil_area || is_masked {
                image.get_pixel(x, y)
            } else {
                image::Rgba([0, 0, 0, 0]) // Transparent pixel
            }
        })
        .into()
    }

    pub fn create_stencil(&self, dimensions: &ImageDimensions) -> DynamicImage {
        let mut stencil_image = create_blank_image(dimensions);
        imageproc::drawing::draw_filled_circle_mut(
            &mut stencil_image,
            dimensions.center_tuple_i32(),
            dimensions.stencil_radius as i32,
            image::Rgba([255, 255, 255, 255]),
        );
        stencil_image
    }

    pub fn create_inverted_stencil(&self, dimensions: &ImageDimensions) -> DynamicImage {
        let mut stencil_image = create_blank_image(dimensions);
        imageproc::drawing::draw_filled_rect_mut(
            &mut stencil_image,
            Rect::at(0, 0).of_size(dimensions.size, dimensions.size),
            image::Rgba([255, 255, 255, 255]),
        );
        imageproc::drawing::draw_filled_circle_mut(
            &mut stencil_image,
            dimensions.center_tuple_i32(),
            dimensions.stencil_radius as i32,
            image::Rgba([0, 0, 0, 0]),
        );
        stencil_image
    }
}
