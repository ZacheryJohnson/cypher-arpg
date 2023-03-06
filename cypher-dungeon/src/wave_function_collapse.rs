use crate::tile::*;
use strum::IntoEnumIterator;

use rand::prelude::*;

pub fn wfc(tiles: &Vec<TileData>) -> Vec<TileInstance> {
    let tile_width = 5;
    let tile_height = 5;

    let tiles_copy = tiles.to_owned();

    {
        let mut domain = vec![tiles_copy; tile_width * tile_height];

        loop {
            if !can_further_collapse(&domain) {
                break;
            }

            if let Some((idx, instance)) = collapse(&mut domain) {
                // instances[idx] = instance;
            } else {
                panic!("failed to complete wfc");
            };
        }

        for y in 0..tile_height {
            for x in 0..tile_width {
                print!("{} ", instances[y * tile_width + x].tile_data.tile_type_id);
            }
            println!();
        }

        println!("Done");
    }

    instances
}

fn can_further_collapse(domain: &Vec<Vec<TileData>>) -> bool {
    domain.iter().any(|possible_tiles| possible_tiles.len() > 1)
}

fn lowest_entropy_idx(domain: &Vec<Vec<TileData>>) -> Option<usize> {
    let (idx, _) = domain
        .iter()
        .enumerate()
        .filter(|(_, possible_tiles)| possible_tiles.len() > 1)
        .min_by(|(_, tiles_a), (_, tiles_b)| tiles_a.len().partial_cmp(&tiles_b.len()).unwrap())?;

    Some(idx)
}

fn collapse(domain: &mut Vec<Vec<TileData>>) -> Option<(usize, TileInstance)> {
    // Get the lowest entropy tile spot in our domain
    let lowest_entropy_idx = lowest_entropy_idx(domain)?;
    let mut tile_set = domain.get_mut(lowest_entropy_idx)?;

    // Collapse the entropy to 1 by picking a random tile
    // Due to lifetimes, we'll generate a random index and remove everything from the vector except that index
    let idx = (0..tile_set.len())
        .choose(&mut rand::thread_rng())
        .unwrap()
        .to_owned();
    *tile_set = tile_set
        .iter()
        .enumerate()
        .filter(|(i, _)| *i == idx)
        .map(|(_, tile)| tile.to_owned())
        .collect();
    let chosen_tile = tile_set.first()?;

    // "Fix" that tile by assigning it a random rotation
    // This is what the game will use
    let rotation = TileRotation::iter()
        .choose(&mut rand::thread_rng())
        .unwrap();
    let fixed_tile = TileInstance {
        tile_data: chosen_tile,
        rotation,
    };

    println!("{fixed_tile:?}");

    let candidate_tiles = tile_set
        .iter()
        .filter(|tile| *tile != chosen_tile)
        .collect::<Vec<&TileData>>();
    let mut compatibilities = vec![];
    for candidate_tile in candidate_tiles {
        compatibilities.extend(TileData::get_compatibilities(&fixed_tile, candidate_tile));
    }
    let random_compat = compatibilities.choose(&mut rand::thread_rng()).unwrap();
    println!(
        "A rotated {:?}; B {:?} rotated {:?}; Type: {:?}",
        random_compat.tile_a_rotation,
        random_compat.tile_b,
        random_compat.tile_b_rotation,
        random_compat.adjacency
    );

    Some((lowest_entropy_idx, fixed_tile))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wfc_tests() {
        let tile0004 = TileData {
            tile_type_id: 4,
            northern_compatibility: 0.into(),
            eastern_compatibility: 5.into(),
            southern_compatibility: 0.into(),
            western_compatibility: 5.into(),
        };

        let tile0006 = TileData {
            tile_type_id: 6,
            northern_compatibility: 10.into(),
            eastern_compatibility: 0.into(),
            southern_compatibility: 10.into(),
            western_compatibility: 0.into(),
        };

        let tile0007 = TileData {
            tile_type_id: 7,
            northern_compatibility: 5.into(),
            eastern_compatibility: 0.into(),
            southern_compatibility: 5.into(),
            western_compatibility: 0.into(),
        };

        let tile0013 = TileData {
            tile_type_id: 13,
            northern_compatibility: 10.into(),
            eastern_compatibility: 11.into(),
            southern_compatibility: 10.into(),
            western_compatibility: 11.into(),
        };

        let tile0049 = TileData {
            tile_type_id: 49,
            northern_compatibility: 0.into(),
            eastern_compatibility: 0.into(),
            southern_compatibility: 0.into(),
            western_compatibility: 0.into(),
        };

        let tileset = vec![tile0004, tile0006, tile0007, tile0013, tile0049];

        let instances = wfc(&tileset);
    }
}
