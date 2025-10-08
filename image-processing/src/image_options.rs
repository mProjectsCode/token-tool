use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ImageTransform {
    pub pos_x: i32,
    pub pos_y: i32,
    pub scale: f32,
    pub flipped: bool,
}

#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ImageDimensions {
    pub size: u32,
    pub oversized: bool,
    pub stencil_radius: u32,
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

    pub fn token_size(&self) -> u32 {
        if self.oversized {
            self.size / 2
        } else {
            self.size
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ImageRenderOptions {
    pub transform: ImageTransform,
    pub dimensions: ImageDimensions,
    pub ring: bool,
}