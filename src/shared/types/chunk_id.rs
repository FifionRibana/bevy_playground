use bevy::prelude::*;
use hexx::Hex;

use crate::hex::{HexConfig, HexCoord};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkId {
    pub coord: HexCoord,
    pub size: u8,
}

impl ChunkId {
    pub fn new(coord: HexCoord, size: u8) -> Self {
        Self { coord, size }
    }
    
    #[inline]
    pub fn from_hex_coord(coord: HexCoord, chunk_size: u8) -> Self {
        let hex = coord.to_hex();

        let chunk_hex = hex.to_lower_res(chunk_size as u32);

        Self {
            coord: HexCoord::from_hex(chunk_hex),
            size: chunk_size,
        }
    }

    pub fn q(&self) -> i32 {
        self.coord.q
    }

    pub fn r(&self) -> i32 {
        self.coord.r
    }

    pub fn distance(&self, other: &ChunkId) -> u32 {
        self.coord.distance(&other.coord)
    }

    pub fn neighbors(&self) -> [ChunkId; 6] {
        let neighbor_hexes = self.coord.neighbors();

        let neighbor_chunks: Vec<ChunkId> = neighbor_hexes
            .iter()
            .map(|h| ChunkId::new(*h, self.size))
            .collect();

        neighbor_chunks.try_into().expect("Wrong neighbors size")
    }

    pub fn range(&self, radius: u32) -> Vec<ChunkId> {
        let range_hexes = self.coord.range(radius);

        range_hexes
            .iter()
            .map(|h| ChunkId::new(*h, self.size))
            .collect()
    }

    pub fn ring(&self, radius: u32) -> Vec<ChunkId> {
        let ring_hexes = self.coord.ring(radius);

        ring_hexes
            .iter()
            .map(|h| ChunkId::new(*h, self.size))
            .collect()
    }

    pub fn rings(&self, range: impl Iterator<Item = u32>) -> Vec<ChunkId> {
        let ring_hexes = self.coord.rings(range);

        ring_hexes
            .iter()
            .map(|h| ChunkId::new(*h, self.size))
            .collect()
    }

    #[inline]
    pub fn from_position(position: Vec2, hex_config: &HexConfig) -> Self {
        let hex_coord = HexCoord::from_hex(hex_config.layout.world_pos_to_hex(position));
        ChunkId::from_hex_coord(hex_coord, hex_config.chunk_size)
    }
}
