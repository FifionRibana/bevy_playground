use crate::hex::HexConfig;
use crate::hex::rendering::mesh;
use bevy::color::palettes::tailwind::{CYAN_300, YELLOW_300};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct ColorTintMaterials {
    pub materials: HashMap<String, Handle<ColorMaterial>>,
    pub hex_mesh: Handle<Mesh>,
}

impl ColorTintMaterials {
    pub fn create(
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        hex_config: Res<HexConfig>,
    ) -> Self {
        // Créer le mesh hexagonal avec hexx
        let hex_mesh = meshes.add(mesh::create_hexagonal_mesh(
            hex_config.layout.clone(),
            hex_config.hex_radius,
        ));

        let mut material_map = HashMap::new();

        let material = materials.add(ColorMaterial::from(Color::srgb_u8(0, 80, 230)));
        material_map.insert("default".to_string(), material);

        let hover_material = materials.add(Color::from(CYAN_300));
        let pressed_material = materials.add(Color::from(YELLOW_300));
        material_map.insert("hover".to_string(), hover_material);
        material_map.insert("pressed".to_string(), pressed_material);

        Self {
            materials: material_map,
            hex_mesh,
        }
    }

    pub fn get_material(&self, name: String) -> Handle<ColorMaterial> {
        self.materials
            .get(&name)
            .cloned()
            .unwrap_or_else(|| self.materials[&"default".to_string()].clone())
    }
}
