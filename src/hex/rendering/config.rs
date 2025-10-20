use bevy::prelude::*;
use hexx::{HexLayout, HexOrientation};

/// Resource wrapper pour la configuration hexagonale
#[derive(Resource, Clone)]
pub struct HexConfig {
    pub layout: HexLayout,
    pub hex_radius: f32,
}

impl Default for HexConfig {
    fn default() -> Self {
        Self::new(48.0, HexOrientation::Flat, Vec2::splat(1.0))
    }
}

impl HexConfig {
    /// Crée une configuration avec un rayon donné
    pub fn new(radius: f32, orientation: HexOrientation, ratio: Vec2) -> Self {
        let mut layout;
        match orientation {
            HexOrientation::Flat => {
                layout = HexLayout::flat();
            }
            HexOrientation::Pointy => {
                layout = HexLayout::pointy();
            }
        }
        layout = layout
            .with_hex_size(radius)
            .with_scale(Vec2::new(ratio[0] * radius, ratio[1] * radius));

        Self {
            layout,
            hex_radius: radius,
        }
    }
}
