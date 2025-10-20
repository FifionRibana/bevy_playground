use crate::hex::HexCoord;
use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct HexVisuals {
    pub tint: Color,
}

impl HexVisuals {
    pub fn new(coord: HexCoord) -> Self {
        // Variation de couleur basée sur coord
        let seed =
            (coord.q as u64).wrapping_mul(374761393) ^ (coord.r as u64).wrapping_mul(668265263);
        let variation = ((seed % 20) as f32 - 10.0) / 100.0; // ±10%

        let tint = Color::srgb(
            (1.0 + variation).clamp(0.0, 1.0),
            (1.0 + variation).clamp(0.0, 1.0),
            (1.0 + variation).clamp(0.0, 1.0),
        );

        Self { tint }
    }
}
