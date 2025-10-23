use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct ContourPath {
    pub points: Vec<Vec2>,
    pub is_closed: bool,
}