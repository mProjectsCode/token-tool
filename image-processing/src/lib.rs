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
    image_options::{ImageDimensions, ImageTransform},
    image_shadow::{ImageShadow, ShadowOptions},
    image_stencil::{ImageStencil, overlay_images},
    utils::set_panic_hook,
};

fn create_blank_image(dimensions: ImageDimensions) -> DynamicImage {
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
        dimensions: ImageDimensions,
        transform: ImageTransform,
        ring: bool,
    ) -> Result<Vec<u8>, JsValue> {
        // Load the image data into a DynamicImage
        let mut image: DynamicImage = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()
            .map_err(|e| JsValue::from_str(&format!("Failed to read image: {e}")))?
            .decode()
            .map_err(|e| JsValue::from_str(&format!("Failed to decode image: {e}")))?;

        let mask: DynamicImage = match mask_data {
            Some(x) => {
                ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(dimensions.size, dimensions.size, x)
                    .ok_or(JsValue::from_str(
                        "Failed to create mask image from provided data",
                    ))?
                    .into()
            }
            None => create_blank_image(dimensions),
        };

        // calc transformed dimensions and offsets
        let (scaled_width, scaled_height) = self.get_scaled_dimensions(&image, transform.scale);

        let x_offset = (dimensions.size as i32 - scaled_width) / 2 + transform.pos_x;
        let y_offset = (dimensions.size as i32 - scaled_height) / 2 + transform.pos_y;

        // flip the image if needed
        if transform.flipped {
            image = image.fliph();
        }
        // resize the image
        image = image.resize(
            scaled_width as u32,
            scaled_height as u32,
            imageops::FilterType::CatmullRom,
        );

        // Crop the image to the correct size
        let mut tmp_image = create_blank_image(dimensions);
        imageops::overlay(&mut tmp_image, &image, x_offset as i64, y_offset as i64);

        let composite_image = self.build_image(&tmp_image, &mask, dimensions, ring)?;

        image_to_bytes(&composite_image)
    }

    pub fn load_border(&mut self, image_data: &[u8], meta: String) -> Result<(), JsValue> {
        self.border = Some(ImageBorder::from_js(image_data, meta)?);

        Ok(())
    }
}

impl ImageProcessor {
    pub fn get_scaled_dimensions(&self, image: &DynamicImage, scale: f32) -> (i32, i32) {
        let scaled_width = (image.width() as f32 * scale) as i32;
        let scaled_height = (image.height() as f32 * scale) as i32;
        (scaled_width, scaled_height)
    }

    pub fn scale_image(&mut self, image: DynamicImage, scale: f32) -> DynamicImage {
        let (scaled_width, scaled_height) = self.get_scaled_dimensions(&image, scale);

        image.resize(
            scaled_width as u32,
            scaled_height as u32,
            imageops::FilterType::CatmullRom,
        )
    }

    pub fn create_ring_image(
        &self,
        dimension: ImageDimensions,
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
        dimensions: ImageDimensions,
        ring: bool,
    ) -> Result<DynamicImage, JsValue> {
        let circle_mask = self.create_stencil(dimensions);
        let circle_mask_inverted = self.create_inverted_stencil(dimensions);
        // circle mask that only keeps the center circle
        let circle_stencil = circle_mask.to_stencil(0);
        // inverted circle mask that keeps everything outside the center circle
        // let circle_stencil_inverted = circle_mask.to_inverted_stencil(0);

        let mask_stencil = mask.to_stencil(0);
        let mask_stencil_inverted = mask.to_inverted_stencil(0);
        let masked_image = image.stencil(&mask_stencil);
        let masked_image_inverted = image.stencil_and(&[&mask_stencil_inverted, &circle_stencil]);

        let ring_image = if ring {
            if let Some(border) = &self.border {
                border.get_ring(dimensions)?
            } else {
                // If no border is loaded, create a default ring image
                self.create_ring_image(dimensions, 20)
            }
        } else {
            (
                create_blank_image(dimensions),
                create_blank_image(dimensions),
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
            dimensions,
            &[
                &ring_image.0,          // the ring image background
                &ring_image.1,          // the ring image foreground
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
        dimensions: ImageDimensions,
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

    pub fn create_stencil(&self, dimensions: ImageDimensions) -> DynamicImage {
        let mut stencil_image = create_blank_image(dimensions);
        imageproc::drawing::draw_filled_circle_mut(
            &mut stencil_image,
            dimensions.center_tuple_i32(),
            dimensions.stencil_radius as i32,
            image::Rgba([255, 255, 255, 255]),
        );
        stencil_image
    }

    pub fn create_inverted_stencil(&self, dimensions: ImageDimensions) -> DynamicImage {
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
