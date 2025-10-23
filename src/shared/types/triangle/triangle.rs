use bevy::math::Vec2;

use crate::shared::types::TriangleId;

// Structures de données
#[derive(Clone, Debug)]
pub struct Triangle {
    pub id: TriangleId,
    pub vertices: [Vec2; 3],
}
