mod utils;

use std::io::Cursor;

use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, ImageReader};
use wasm_bindgen::{prelude::*};

fn create_blank_image(dimensions: ImageDimensions) -> DynamicImage {
    let blank_image = image::ImageBuffer::new(dimensions.size, dimensions.size);
    DynamicImage::ImageRgba8(blank_image)
}

fn image_to_bytes(image: &DynamicImage) -> Result<Vec<u8>, JsValue> {
    let mut bytes = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::WebP)
        .map_err(|e| JsValue::from_str(&format!("Failed to write image: {}", e)))?;
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
pub struct ImageProcessor {
    mask_image: Option<DynamicImage>,
}

#[wasm_bindgen]
impl ImageProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        let mask_image = None;

        Ok(ImageProcessor { mask_image })
    }

    pub fn clear_mask(&mut self, dimensions: ImageDimensions) -> Result<Vec<u8>, JsValue> {
        self.mask_image = Some(create_blank_image(dimensions));

        image_to_bytes(&self.mask_image.as_ref().unwrap())
    }

    pub fn set_mask(&mut self, mask_data: &[u8], dimensions: ImageDimensions) -> Result<(), JsValue> {
        let mask_image = ImageReader::new(Cursor::new(mask_data))
            .with_guessed_format()
            .map_err(|e| JsValue::from_str(&format!("Failed to read mask image: {}", e)))?
            .decode()
            .map_err(|e| JsValue::from_str(&format!("Failed to decode mask image: {}", e)))?;

        if mask_image.width() != dimensions.size || mask_image.height() != dimensions.size {
            return Err(JsValue::from_str("Mask image size does not match dimensions"));
        }

        self.mask_image = Some(mask_image);
        Ok(())
    }

    pub fn draw_on_mask(&mut self, size: u32, add: bool, x: i32, y: i32) -> Result<Vec<u8>, JsValue> {
        if size == 0 {
            return Err(JsValue::from_str("Size must be greater than 0"));
        }

        let radius = size as i32 / 2;
        let color = if add { image::Rgba([0, 255, 0, 40]) } else { image::Rgba([0, 0, 0, 0]) };

        let Some(img) = &mut self.mask_image else {
            return Err(JsValue::from_str("Mask image is not set"));
        };

        imageproc::drawing::draw_filled_circle_mut(
            img,
            (x, y),
            radius,
            color,
        );

        image_to_bytes(img)
    }

    pub fn render(&mut self, image_data: &[u8], dimensions: ImageDimensions, transform: ImageTransform, ring: bool) -> Result<Vec<u8>, JsValue> {
        
        // Load the image data into a DynamicImage
        let mut image: DynamicImage = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()
            .map_err(|e| JsValue::from_str(&format!("Failed to read image: {}", e)))?
            .decode()
            .map_err(|e| JsValue::from_str(&format!("Failed to decode image: {}", e)))?;
        
        // calc transformed dimensions and offsets
        let (scaled_width, scaled_height) = self.get_scaled_dimensions(&image, transform.scale);
        
        let x_offset = (dimensions.size as i32 - scaled_width) / 2 + transform.pos_x;
        let y_offset = (dimensions.size as i32 - scaled_height) / 2 + transform.pos_y;
        
        // flip the image if needed
        if transform.flipped {
            image = image.fliph();
        }
        // resize the image
        image = image.resize(scaled_width as u32, scaled_height as u32, imageops::FilterType::CatmullRom);
        
        // Crop the image to the correct size
        let mut tmp_image = create_blank_image(dimensions);
        image::imageops::overlay(&mut tmp_image, &image, x_offset as i64, y_offset as i64);

        // Stencil the image
        let stenciled_image = ImageBuffer::from_fn(dimensions.size, dimensions.size, |x, y| {
            let distance_to_center = (x as i32 - (dimensions.size / 2) as i32).pow(2) + (y as i32 - (dimensions.size / 2) as i32).pow(2);
            let in_stencil_area = distance_to_center <= dimensions.stencil_radius.pow(2) as i32;
            let is_masked = self.mask_image.as_ref().map_or(false, |mask| mask.get_pixel(x, y).0[3] > 0);

            if in_stencil_area || is_masked {
                tmp_image.get_pixel(x, y)
            } else {
                image::Rgba([0, 0, 0, 0]) // Transparent pixel
            }
        });

        // composite the final image
        let mut composite_image = create_blank_image(dimensions);
        
        if ring {
            let ring_image = self.generate_ring_image(dimensions, 20);
            image::imageops::overlay(&mut composite_image, &ring_image, 0, 0);
        }
        image::imageops::overlay(
            &mut composite_image,
            &stenciled_image,
            0,
            0,
        );

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

        return image.resize(scaled_width as u32, scaled_height as u32, imageops::FilterType::CatmullRom);
    }

    pub fn generate_ring_image(&self, dimension: ImageDimensions, ring_width: u32) -> DynamicImage {
        let mut ring_image = create_blank_image(dimension);
        imageproc::drawing::draw_filled_circle_mut(
            &mut ring_image,
            dimension.center_tuple_i32(),
            (dimension.stencil_radius + ring_width) as i32,
            image::Rgba([255, 255, 255, 255]),
        );
        imageproc::drawing::draw_filled_circle_mut(
            &mut ring_image,
            dimension.center_tuple_i32(),
            dimension.stencil_radius as i32,
            image::Rgba([0, 0, 0, 0]),
        );

        ring_image
    }
}