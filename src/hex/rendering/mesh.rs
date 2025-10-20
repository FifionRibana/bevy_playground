use bevy::prelude::*;
use bevy::mesh::Indices;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use hexx::{PlaneMeshBuilder, HexLayout, MeshInfo};

/// Crée un mesh hexagonal en utilisant hexx::ColumnMeshBuilder
pub fn create_hexagonal_mesh(layout: HexLayout, radius: f32) -> Mesh {
    // Utilise ColumnMeshBuilder de hexx pour un hexagone plat
    let mesh_info = PlaneMeshBuilder::new(&layout)
        .facing(Vec3::Z)
        .center_aligned()
        .build();
    
    hexagonal_mesh(mesh_info)
}

/// Convertit hexx::MeshInfo en bevy::Mesh
/// Source: https://docs.rs/hexx/latest/hexx/#bevy-integration
pub fn hexagonal_mesh(mesh_info: MeshInfo) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
    .with_inserted_indices(Indices::U16(mesh_info.indices))
}
