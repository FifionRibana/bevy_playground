use bevy::prelude::*;

#[derive(Clone, Debug)]
pub enum GenerationStage {
    Idle,
    LoadingImage,
    SamplingTerrain,
    GeneratingContours,
    SmoothingContours,
    TriangulatingMesh,
    Complete,
}
