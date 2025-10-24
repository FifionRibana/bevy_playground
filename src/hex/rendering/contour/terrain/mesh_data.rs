// Structure pour passer les données entre threads
#[derive(Clone)]
pub struct TerrainMeshData {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u16>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}