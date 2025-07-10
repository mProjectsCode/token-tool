use std::{collections::HashMap, io::Cursor};

use hex_color::HexColor;
use image::{
    DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba,
    imageops::{self, FilterType},
};
use serde::Deserialize;
use wasm_bindgen::JsValue;

use crate::{create_blank_image, image_options::ImageDimensions};

pub struct ImageBorder {
    sprite_sheet: DynamicImage,
    config: RingConfig,
    ring_frames: Vec<RingFrame>,
    bkg_frames: Vec<BkgFrame>,
}

impl ImageBorder {
    pub fn from_js(img: &[u8], config: String) -> Result<Self, JsValue> {
        let image: DynamicImage = ImageReader::new(Cursor::new(img))
            .with_guessed_format()
            .map_err(|e| JsValue::from_str(&format!("Failed to read image: {e}")))?
            .decode()
            .map_err(|e| JsValue::from_str(&format!("Failed to decode image: {e}")))?;

        let config: SpriteSheetConfig = serde_json::from_str(&config)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {e}")))?;

        let mut ring_frames: Vec<RingFrame> = config
            .frames
            .values()
            .filter_map(|frame| {
                if let Frame::Ring(ring_frame) = frame {
                    Some(ring_frame.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut bkg_frames: Vec<BkgFrame> = config
            .frames
            .values()
            .filter_map(|frame| {
                if let Frame::Bkg(bkg_frame) = frame {
                    Some(bkg_frame.clone())
                } else {
                    None
                }
            })
            .collect();

        ring_frames.sort_by_key(|frame| frame.frame.width);
        bkg_frames.sort_by_key(|frame| frame.frame.width);

        Ok(ImageBorder {
            sprite_sheet: image,
            config: config.config,
            ring_frames,
            bkg_frames,
        })
    }

    pub fn get_ring(
        &self,
        dimensions: ImageDimensions,
    ) -> Result<(DynamicImage, DynamicImage), JsValue> {
        let (bkg_frame, ring_frame) = self.get_frame_for_dimensions(dimensions)?;

        let bkg_image = self.cut_and_scale_bkg(bkg_frame, dimensions);
        let ring_image = self.cut_and_scale_ring(ring_frame, dimensions)?;

        Ok((bkg_image, ring_image))
    }

    fn get_frame_for_dimensions(
        &self,
        dimensions: ImageDimensions,
    ) -> Result<(&BkgFrame, &RingFrame), JsValue> {
        let best_bkg_frame = self
            .bkg_frames
            .iter()
            .find(|frame| frame.frame.width >= dimensions.size);
        let best_ring_frame = self
            .ring_frames
            .iter()
            .find(|frame| frame.frame.width >= dimensions.size);

        match (best_bkg_frame, best_ring_frame) {
            (Some(bkg_frame), Some(ring_frame)) => Ok((bkg_frame, ring_frame)),
            _ => Err(JsValue::from_str(&format!(
                "No suitable ring ({}) or background frame ({}) found for the given dimensions ({})",
                self.ring_frames.len(),
                self.bkg_frames.len(),
                dimensions.size
            ))),
        }
    }

    fn cut_and_scale_bkg(&self, frame: &BkgFrame, dimensions: ImageDimensions) -> DynamicImage {
        let cut_image = self.sprite_sheet.crop_imm(
            frame.frame.x as u32,
            frame.frame.y as u32,
            frame.frame.width,
            frame.frame.height,
        );

        self.scale_img(cut_image, dimensions)
    }

    fn cut_and_scale_ring(
        &self,
        frame: &RingFrame,
        dimensions: ImageDimensions,
    ) -> Result<DynamicImage, JsValue> {
        let cut_image = self.sprite_sheet.crop_imm(
            frame.frame.x as u32,
            frame.frame.y as u32,
            frame.frame.width,
            frame.frame.height,
        );

        let band_color = HexColor::parse(&self.config.default_ring_color)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse default ring color: {e}")))?;

        let cut_image: DynamicImage =
            ImageBuffer::from_fn(cut_image.width(), cut_image.height(), |x, y| {
                let pixel = cut_image.get_pixel(x, y);
                let in_color_band = {
                    let squared_distance_to_center = (x as i32 - cut_image.width() as i32 / 2)
                        .pow(2)
                        + (y as i32 - cut_image.height() as i32 / 2).pow(2);
                    let start_radius =
                        frame.color_band.start_radius * (cut_image.width() as f32 / 2.0);
                    let end_radius = frame.color_band.end_radius * (cut_image.width() as f32 / 2.0);
                    squared_distance_to_center >= start_radius.powf(2.0) as i32
                        && squared_distance_to_center <= end_radius.powf(2.0) as i32
                };

                if in_color_band {
                    // We simply take the R channel as albedo.
                    // Not sure if this is correct.
                    let albedo = pixel[0] as f32 / 255.0;
                    let color = band_color.scale(albedo);

                    Rgba([color.r, color.g, color.b, pixel[3]])
                } else {
                    pixel
                }
            })
            .into();

        Ok(self.scale_img(cut_image, dimensions))
    }

    fn scale_img(&self, image: DynamicImage, dimensions: ImageDimensions) -> DynamicImage {
        let token_size = if dimensions.oversized {
            dimensions.size / 2
        } else {
            dimensions.size
        };

        let scaled_img = if image.width() == token_size && image.height() == token_size {
            image
        } else {
            image.resize(token_size, token_size, FilterType::CatmullRom)
        };

        if dimensions.oversized {
            let mut blank_image = create_blank_image(dimensions);
            imageops::overlay(
                &mut blank_image,
                &scaled_img,
                (dimensions.size / 4) as i64,
                (dimensions.size / 4) as i64,
            );
            blank_image
        } else {
            scaled_img
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Point {
    #[serde(alias = "w")]
    pub x: i32,
    #[serde(alias = "h")]
    pub y: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct FloatPoint {
    #[serde(alias = "w")]
    pub x: f32,
    #[serde(alias = "h")]
    pub y: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct Box {
    pub x: i32,
    pub y: i32,
    #[serde(rename = "w")]
    pub width: u32,
    #[serde(rename = "h")]
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct ColorBand {
    #[serde(rename = "startRadius")]
    pub start_radius: f32,
    #[serde(rename = "endRadius")]
    pub end_radius: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct BkgFrame {
    pub frame: Box,
    pub rotated: bool,
    pub trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    pub sprite_source_size: Box,
    #[serde(rename = "sourceSize")]
    pub source_size: Point,
    pub anchor: FloatPoint,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct RingFrame {
    pub frame: Box,
    pub rotated: bool,
    pub trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    pub sprite_source_size: Box,
    #[serde(rename = "sourceSize")]
    pub source_size: Point,
    pub anchor: FloatPoint,
    #[serde(rename = "gridTarget")]
    pub grid_target: u32,
    #[serde(rename = "colorBand")]
    pub color_band: ColorBand,
    #[serde(rename = "ringThickness")]
    pub ring_thickness: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(unused)]
pub enum Frame {
    Ring(RingFrame),
    Bkg(BkgFrame),
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct RingConfig {
    #[serde(rename = "defaultColorBand")]
    default_color_band: ColorBand,
    #[serde(rename = "defaultRingColor")]
    default_ring_color: String, // Hex color code
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct SpriteSheetConfig {
    pub config: RingConfig,
    pub frames: HashMap<String, Frame>,
}
