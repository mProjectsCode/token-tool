mod utils;

use std::io::Cursor;

use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba, imageops};
use imageproc::rect::Rect;
use wasm_bindgen::prelude::*;

use crate::utils::set_panic_hook;

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
pub struct ImageTransform {
    pos_x: i32,
    pos_y: i32,
    scale: f32,
    flipped: bool,
}

#[wasm_bindgen]
impl ImageTransform {
    #[wasm_bindgen(constructor)]
    pub fn new(pos_x: i32, pos_y: i32, scale: f32, flipped: bool) -> Self {
        ImageTransform {
            pos_x,
            pos_y,
            scale,
            flipped,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct ImageDimensions {
    size: u32,
    stencil_radius: u32,
}

#[wasm_bindgen]
impl ImageDimensions {
    #[wasm_bindgen(constructor)]
    pub fn new(size: u32, stencil_radius: u32) -> Self {
        ImageDimensions {
            size,
            stencil_radius,
        }
    }
}

impl ImageDimensions {
    pub fn center(&self) -> u32 {
        self.size / 2
    }

    pub fn center_tuple(&self) -> (u32, u32) {
        (self.center(), self.center())
    }

    pub fn center_i32(&self) -> i32 {
        self.size as i32 / 2
    }

    pub fn center_tuple_i32(&self) -> (i32, i32) {
        (self.center_i32(), self.center_i32())
    }
}

#[wasm_bindgen]
pub struct ImageProcessor {}

#[wasm_bindgen]
impl ImageProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        set_panic_hook();

        Ok(ImageProcessor {})
    }

    pub fn render(
        &mut self,
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

        let composite_image = self.build_image(&tmp_image, &mask, dimensions, ring);

        image_to_bytes(&composite_image)
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

    pub fn create_ring_image(&self, dimension: ImageDimensions, ring_width: u32) -> DynamicImage {
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

        ring_image
    }

    pub fn stencil(
        &self,
        image: &DynamicImage,
        mask: &DynamicImage,
        dimensions: ImageDimensions,
        invert: bool,
        threshold: u8,
    ) -> DynamicImage {
        ImageBuffer::from_fn(dimensions.size, dimensions.size, |x, y| {
            let mask_pixel = mask.get_pixel(x, y);
            if (mask_pixel[3] > threshold) ^ invert {
                // Check alpha channel
                image.get_pixel(x, y)
            } else {
                image::Rgba([0, 0, 0, 0]) // Transparent pixel
            }
        })
        .into()
    }

    pub fn to_shadow(&self, image: &DynamicImage, dimensions: ImageDimensions) -> DynamicImage {
        let image = ImageBuffer::from_fn(dimensions.size, dimensions.size, |x, y| {
            let pixel = image.get_pixel(x, y);
            image::Rgba([0, 0, 0, (pixel[3] as f32 * 0.5) as u8])
        });

        imageops::blur(&image, 15.0).into()
    }

    pub fn build_image(
        &self,
        image: &DynamicImage,
        mask: &DynamicImage,
        dimensions: ImageDimensions,
        ring: bool,
    ) -> DynamicImage {
        let stencil = self.create_stencil(dimensions);
        let inverted_stencil = self.create_inverted_stencil(dimensions);
        let mut composite_image = create_blank_image(dimensions);

        if ring {
            let ring_image = self.create_ring_image(dimensions, 20);
            imageops::overlay(&mut composite_image, &ring_image, 0, 0);
        }

        let stenciled_image_shadow = self.stencil(image, mask, dimensions, false, 0);
        let image_shadow = self.to_shadow(&stenciled_image_shadow, dimensions);
        let image_shadow = self.stencil(&image_shadow, &inverted_stencil, dimensions, false, 0);

        imageops::overlay(&mut composite_image, &image_shadow, 0, 0);

        let masked_and_stenciled_image = self.mask_and_stencil_image(image, mask, dimensions);


        let masked_inverted_stencil = self.stencil(&inverted_stencil, mask, dimensions, true, 0);
        let ring_shadow = self.to_shadow(&masked_inverted_stencil, dimensions);
        let stenciled_ring_shadow = self.stencil(&ring_shadow, &stencil, dimensions, false, 0);
        let stenciled_ring_shadow = self.stencil(&stenciled_ring_shadow, mask, dimensions, true, 0);
        let stenciled_ring_shadow = self.stencil(&stenciled_ring_shadow, &masked_and_stenciled_image, dimensions, false, 10);

        imageops::overlay(&mut composite_image, &masked_and_stenciled_image, 0, 0);
        imageops::overlay(&mut composite_image, &stenciled_ring_shadow, 0, 0);

        composite_image
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
