use bevy::prelude::*;

use crate::hex::HexCoord;

#[derive(Component)]
pub struct HexTile {
    pub coord: HexCoord,
}
