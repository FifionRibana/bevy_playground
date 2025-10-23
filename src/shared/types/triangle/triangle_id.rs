use hexx::*;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct TriangleId {
    pub hex: Hex,
    pub index: usize,
}