use super::ColorTintMaterials;
use crate::hex::HexConfig;
use bevy::prelude::*;

/// Setup des matériaux de biomes (APRÈS HexConfig)
pub fn setup_biome_materials(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
    hex_config: Res<HexConfig>,
) {

    let color_tint_materials = ColorTintMaterials::create(meshes, materials, hex_config);
    commands.insert_resource(color_tint_materials);
    info!("✓ Color tint materials créés");
}
