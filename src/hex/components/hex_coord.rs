use bevy::prelude::*;
use hexx::Hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Convertit en Hex de la librairie hexx
    #[inline]
    pub fn to_hex(&self) -> Hex {
        Hex::new(self.q, self.r)
    }

    /// Crée depuis un Hex de la librairie hexx
    #[inline]
    pub fn from_hex(hex: Hex) -> Self {
        Self { q: hex.x, r: hex.y }
    }

    /// Retourne les 6 voisins (utilise hexx)
    pub fn neighbors(&self) -> [HexCoord; 6] {
        let hex = self.to_hex();
        hex.all_neighbors().map(HexCoord::from_hex)
    }

    /// Distance Manhattan entre deux hexagones (utilise hexx)
    pub fn distance(&self, other: &HexCoord) -> u32 {
        self.to_hex().unsigned_distance_to(other.to_hex())
    }

    /// Calcule tous les hexagones dans un rayon (utilise hexx)
    pub fn in_range(&self, range: u32) -> Vec<HexCoord> {
        self.to_hex().range(range).map(HexCoord::from_hex).collect()
    }

    /// Trace une ligne vers un autre hex (utilise hexx)
    pub fn line_to(&self, other: &HexCoord) -> Vec<HexCoord> {
        self.to_hex()
            .line_to(other.to_hex())
            .map(HexCoord::from_hex)
            .collect()
    }

    pub fn range(&self, radius: u32) -> Vec<HexCoord> {
        self.to_hex().range(radius).map(HexCoord::from_hex).collect()

    }

    /// Retourne un anneau d'hexagones à une distance donnée
    pub fn ring(&self, radius: u32) -> Vec<HexCoord> {
        self.to_hex().ring(radius).map(HexCoord::from_hex).collect()
    }

    /// Retourne un anneau d'hexagones à une distance donnée
    pub fn rings(&self, range: impl Iterator<Item = u32>) -> Vec<HexCoord> {
        self.to_hex().rings(range).into_iter().flatten().map(HexCoord::from_hex).collect()
    }
    
    /// Retourne un anneau d'hexagones à une distance donnée
    pub fn spiral(&self, range: impl Iterator<Item = u32>) -> Vec<HexCoord> {
        
        self.to_hex().rings(range).into_iter().flatten().map(HexCoord::from_hex).collect()
    }
}
