use std::ops::Shl;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug)]
pub struct TileCompatability {
    pub left: u8,
    pub middle: u8,
    pub right: u8,
    // unused - room for another u8 if we wish
}

impl TileCompatability {
    pub fn new(left: u8, middle: u8, right: u8) -> TileCompatability {
        TileCompatability {
            left,
            middle,
            right,
        }
    }
}

impl PartialEq for TileCompatability {
    fn eq(&self, other: &Self) -> bool {
        Into::<u32>::into(*self) == Into::<u32>::into(*other)
    }
}

impl From<u32> for TileCompatability {
    fn from(value: u32) -> Self {
        TileCompatability {
            left: ((value & 0xFF0000) >> 16) as u8,
            middle: ((value & 0xFF00) >> 8) as u8,
            right: (value & 0xFF) as u8,
        }
    }
}

impl From<TileCompatability> for u32 {
    fn from(value: TileCompatability) -> Self {
        (value.left as u32).shl(16) | (value.middle as u32).shl(8) | (value.right as u32)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TileData {
    pub tile_type_id: u32,

    // Compatibilities for wave function collapse
    pub northern_compatibility: TileCompatability,
    pub eastern_compatibility: TileCompatability,
    pub southern_compatibility: TileCompatability,
    pub western_compatibility: TileCompatability,
}

#[derive(Debug)]
pub struct TileInstance<'data> {
    pub tile_data: &'data TileData,
    pub rotation: TileRotation,
}

#[derive(Clone, Copy, EnumIter, Debug, PartialEq)]
pub enum TileSide {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, EnumIter, Debug, PartialEq)]
pub enum TileRotation {
    Identity,         // 0 degrees of rotation
    Clockwise,        // 90 degrees of rotation
    Opposite,         // 180 degrees of rotation
    Counterclockwise, // 270 degrees of rotation
}

#[derive(Clone, Copy, EnumIter, Debug, PartialEq)]
pub enum TileAdjacencyType {
    NorthSouth,
    EastWest,
    SouthNorth,
    WestEast,
}

#[derive(Debug)]
pub struct TileAdjacency<'a, 'b> {
    pub tile_a: &'a TileData,
    pub tile_a_rotation: TileRotation,
    pub tile_b: &'b TileData,
    pub tile_b_rotation: TileRotation,
    pub adjacency: TileAdjacencyType,
}

impl TileData {
    pub fn get_compatibilities<'a, 'b>(
        fixed_tile: &'a TileInstance,
        potential_tile: &'b TileData,
    ) -> Vec<TileAdjacency<'a, 'b>> {
        let mut adjacencies = vec![];

        for potential_rotation in TileRotation::iter() {
            for adjacency_type in TileAdjacencyType::iter() {
                if !TileData::is_compatible(
                    &fixed_tile.tile_data,
                    fixed_tile.rotation,
                    potential_tile,
                    potential_rotation,
                    adjacency_type,
                ) {
                    continue;
                }

                adjacencies.push(TileAdjacency {
                    tile_a: &fixed_tile.tile_data,
                    tile_a_rotation: fixed_tile.rotation,
                    tile_b: potential_tile,
                    tile_b_rotation: potential_rotation,
                    adjacency: adjacency_type,
                });
            }
        }

        adjacencies
    }

    fn is_compatible(
        tile_a: &TileData,
        tile_a_rotation: TileRotation,
        tile_b: &TileData,
        tile_b_rotation: TileRotation,
        tile_adjacency_type: TileAdjacencyType,
    ) -> bool {
        match tile_adjacency_type {
            TileAdjacencyType::NorthSouth => {
                let a_compat =
                    TileData::get_compatibility(tile_a, tile_a_rotation, TileSide::South);
                let b_compat =
                    TileData::get_compatibility(tile_b, tile_b_rotation, TileSide::North);
                a_compat == b_compat
            }
            TileAdjacencyType::EastWest => {
                let a_compat = TileData::get_compatibility(tile_a, tile_a_rotation, TileSide::West);
                let b_compat = TileData::get_compatibility(tile_b, tile_b_rotation, TileSide::East);
                a_compat == b_compat
            }
            TileAdjacencyType::SouthNorth => {
                let a_compat =
                    TileData::get_compatibility(tile_a, tile_a_rotation, TileSide::North);
                let b_compat =
                    TileData::get_compatibility(tile_b, tile_b_rotation, TileSide::South);
                a_compat == b_compat
            }
            TileAdjacencyType::WestEast => {
                let a_compat = TileData::get_compatibility(tile_a, tile_a_rotation, TileSide::East);
                let b_compat = TileData::get_compatibility(tile_b, tile_b_rotation, TileSide::West);
                a_compat == b_compat
            }
        }
    }

    fn get_compatibility(
        tile: &TileData,
        rotation: TileRotation,
        side: TileSide,
    ) -> TileCompatability {
        match (rotation, side) {
            (TileRotation::Identity, TileSide::North) => tile.northern_compatibility,
            (TileRotation::Identity, TileSide::East) => tile.eastern_compatibility,
            (TileRotation::Identity, TileSide::South) => tile.southern_compatibility,
            (TileRotation::Identity, TileSide::West) => tile.western_compatibility,
            (TileRotation::Clockwise, TileSide::North) => tile.western_compatibility,
            (TileRotation::Clockwise, TileSide::East) => tile.northern_compatibility,
            (TileRotation::Clockwise, TileSide::South) => tile.eastern_compatibility,
            (TileRotation::Clockwise, TileSide::West) => tile.southern_compatibility,
            (TileRotation::Opposite, TileSide::North) => tile.southern_compatibility,
            (TileRotation::Opposite, TileSide::East) => tile.western_compatibility,
            (TileRotation::Opposite, TileSide::South) => tile.northern_compatibility,
            (TileRotation::Opposite, TileSide::West) => tile.eastern_compatibility,
            (TileRotation::Counterclockwise, TileSide::North) => tile.eastern_compatibility,
            (TileRotation::Counterclockwise, TileSide::East) => tile.southern_compatibility,
            (TileRotation::Counterclockwise, TileSide::South) => tile.western_compatibility,
            (TileRotation::Counterclockwise, TileSide::West) => tile.northern_compatibility,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn testing() {
        let scoped_tile_data = TileData {
            tile_type_id: 1,
            northern_compatibility: 1.into(),
            eastern_compatibility: 1.into(),
            southern_compatibility: 1.into(),
            western_compatibility: 1.into(),
        };

        let fixed_tile = TileInstance {
            tile_data: &scoped_tile_data,
            rotation: TileRotation::Identity,
        };

        let tile_b = TileData {
            tile_type_id: 2,
            northern_compatibility: 0.into(),
            eastern_compatibility: 0.into(),
            southern_compatibility: 1.into(),
            western_compatibility: 2.into(),
        };

        let compats = TileData::get_compatibilities(&fixed_tile, &tile_b);
        for c in compats {
            println!(
                "A rotated {:?}; B rotated {:?}; Type: {:?}",
                c.tile_a_rotation, c.tile_b_rotation, c.adjacency
            );
        }
    }
}
