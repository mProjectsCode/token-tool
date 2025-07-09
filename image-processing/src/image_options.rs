use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ImageTransform {
    pub pos_x: i32,
    pub pos_y: i32,
    pub scale: f32,
    pub flipped: bool,
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
    pub size: u32,
    pub stencil_radius: u32,
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
