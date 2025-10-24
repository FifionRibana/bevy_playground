use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use futures_lite as future;

use super::super::terrain::{GenerationProgressHandle, TerrainGenerationTask};

pub fn process_terrain_generation(
    mut commands: Commands,
    mut task: Option<ResMut<TerrainGenerationTask>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    progress_handle: Option<Res<GenerationProgressHandle>>,
) {
    // Mettre à jour le progress depuis le handle thread-safe
    if let Some(handle) = progress_handle {
        if let Ok(current_progress) = handle.0.lock() {
            commands.insert_resource(current_progress.clone());
        }
    }

    // Vérifier si la tâche est terminée
    if let Some(mut task) = task {
        if let Some(mesh_data) = future::block_on(future::poll_once(&mut task.0)) {
            // Créer le mesh Bevy
            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs)
            .with_inserted_indices(Indices::U16(mesh_data.indices));

            // Spawner l'entité
            commands.spawn((
                Mesh2d(meshes.add(mesh)),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.4, 0.6, 0.3)))),
            ));

            // Retirer la tâche
            commands.remove_resource::<TerrainGenerationTask>();
        }
    }
}
