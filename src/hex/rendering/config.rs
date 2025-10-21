use bevy::prelude::*;
use hexx::{HexLayout, HexOrientation};

/// Resource wrapper pour la configuration hexagonale
#[derive(Resource, Clone)]
pub struct HexConfig {
    pub layout: HexLayout,
    pub chunk_layout: HexLayout,
    pub hex_radius: f32,
    pub chunk_size: u8,
}

impl Default for HexConfig {
    fn default() -> Self {
        Self::new(48.0, HexOrientation::Flat, Vec2::splat(1.0), 8)
    }
}

impl HexConfig {
    /// Crée une configuration avec un rayon donné
    pub fn new(radius: f32, orientation: HexOrientation, ratio: Vec2, chunk_size: u8) -> Self {
        let mut layout;
        let mut chunk_layout;
        match orientation {
            HexOrientation::Flat => {
                layout = HexLayout::flat();
                chunk_layout = HexLayout::pointy();
            }
            HexOrientation::Pointy => {
                layout = HexLayout::pointy();
                chunk_layout = HexLayout::flat();
            }
        }
        layout = layout
            .with_hex_size(radius)
            .with_scale(Vec2::new(ratio[0] * radius, ratio[1] * radius));

        chunk_layout = chunk_layout
            .with_hex_size(chunk_size as f32 * radius)
            .with_scale(Vec2::splat(chunk_size as f32) * layout.scale);

        Self {
            layout,
            chunk_layout,
            hex_radius: radius,
            chunk_size,
        }
    }
}
