use bevy::prelude::*;
use hexx::Hex;

use super::hex_coord::HexCoord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexChunk {
    pub q: i32,
    pub r: i32,
    pub tiles: Vec<HexCoord>
}

impl HexChunk {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r, Vec::new() }
    }