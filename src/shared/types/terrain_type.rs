use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TerrainType {
    DeepWater,    // Loin de la côte
    ShallowWater, // Proche de la côte
    Beach,        // Frontière côté mer
    Cliff,        // Frontière côté terre (si élévation)
    Land,         // Terre ferme
}