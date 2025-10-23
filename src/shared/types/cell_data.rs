use super::TerrainType;
use crate::hex::components::HexCoord;

#[derive(Clone)]
pub struct CellData {
    pub coord: HexCoord,
    pub terrain_type: TerrainType,
    pub is_border: bool,
    // Valeur d'échantillonnage de la binary map (0.0 = noir, 1.0 = blanc)
    pub sample_value: f32,
    // Distance au bord le plus proche
    pub distance_to_edge: f32,
}
