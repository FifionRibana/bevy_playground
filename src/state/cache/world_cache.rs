use bevy::prelude::*;
use std::collections::HashSet;

use crate::shared::types::ChunkId;

#[derive(Resource, Default)]
pub struct WorldCache {
    pub chunks: HashSet<ChunkId>,
    pub requested_chunks: HashSet<ChunkId>,
    pub unload_chunks_request: HashSet<ChunkId>,
    
}

impl WorldCache {
    pub fn insert_chunk(
        &mut self,
        chunk_id: ChunkId
    ) {
        self.chunks.insert(chunk_id);
        self.requested_chunks.remove(&chunk_id);
    }

    pub fn is_loaded(&self, chunk_id: &ChunkId) -> bool {
        self.chunks.contains(chunk_id)
    }

    pub fn is_requested(&self, chunk_id: &ChunkId) -> bool {
        self.requested_chunks.contains(chunk_id)
    }

    pub fn mark_requested(&mut self, chunk_id: ChunkId) {
        self.requested_chunks.insert(chunk_id);
    }

    pub fn unload_distant(&mut self, center: ChunkId, max_distance: u32) -> Vec<ChunkId> {

        self.chunks.retain(|chunk_id| {
            let keep = chunk_id.distance(&center) as u32 <= max_distance + 1;

            if !keep {
                self.unload_chunks_request.insert(*chunk_id);
            }

            keep
        });

        if !self.unload_chunks_request.is_empty() {
            warn!("📦 Unloaded {} chunks: {:?} from world cache", self.unload_chunks_request.len(), self.unload_chunks_request)
        }
        self.unload_chunks_request.clone().into_iter().collect()
    }

    pub fn unload_chunk(&mut self, chunk_id: ChunkId) {
        self.unload_chunks_request.remove(&chunk_id);
    }
}