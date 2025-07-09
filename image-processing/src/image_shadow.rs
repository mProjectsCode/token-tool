use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, imageops};

pub trait ImageShadow {
    fn to_shadow(&self, options: &ShadowOptions) -> DynamicImage;
}

impl ImageShadow for DynamicImage {
    fn to_shadow(&self, options: &ShadowOptions) -> DynamicImage {
        let img = ImageBuffer::from_fn(self.width(), self.height(), |x, y| {
            let (offset_x, offset_y) = (x as i32 + options.offset_x, y as i32 + options.offset_y);

            let offset_in_bounds = offset_x >= 0
                && offset_x < self.width() as i32
                && offset_y >= 0
                && offset_y < self.height() as i32;

            let opacity = if offset_in_bounds {
                self.get_pixel(offset_x as u32, offset_y as u32)[3] as f32
            } else {
                0.0
            };

            image::Rgba([
                options.color[0],
                options.color[1],
                options.color[2],
                (opacity * options.opacity) as u8,
            ])
        });

        imageops::blur(&img, options.blur).into()
    }
}

#[derive(Debug, Clone)]
pub struct ShadowOptions {
    pub color: Rgb<u8>,
    /// Opacity between 0.0 and 1.0
    /// 0.0 means fully transparent, 1.0 means fully opaque
    /// Recommended value is 0.4
    pub opacity: f32,
    /// Blur radius for the shadow
    /// Recommended value is 5.0
    pub blur: f32,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl ShadowOptions {
    pub fn new(color: Rgb<u8>, opacity: f32, blur: f32, offset_x: i32, offset_y: i32) -> Self {
        ShadowOptions {
            color,
            opacity,
            blur,
            offset_x,
            offset_y,
        }
    }

    pub fn new_black(opacity: f32, blur: f32, offset_x: i32, offset_y: i32) -> Self {
        ShadowOptions::new(Rgb([0, 0, 0]), opacity, blur, offset_x, offset_y)
    }
}
