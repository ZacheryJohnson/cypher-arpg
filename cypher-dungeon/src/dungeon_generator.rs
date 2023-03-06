#[derive(Resource)]
pub struct DungeonGenerator {
    pub seed: u128,
}

impl Default for DungeonGenerator {
    fn default() -> Self {
        let u32s = vec![
            thread_rng().next_u32(),
            thread_rng().next_u32(),
            thread_rng().next_u32(),
            thread_rng().next_u32(),
        ];

        let seed: u128 = u32s
            .into_iter()
            .fold(0u128, |acc, next| acc << 8 | (next as u128));

        DungeonGenerator { seed }
    }
}

pub struct DungeonGenerationSettings {
    pub min_x_size: u16,
    pub max_x_size: u16,
    pub min_y_size: u16,
    pub max_y_size: u16,
    pub min_rooms: u8,
    pub max_rooms: u8,
}

impl Default for DungeonGenerationSettings {
    fn default() -> Self {
        DungeonGenerationSettings {
            min_x_size: 30,
            max_x_size: 50,
            min_y_size: 30,
            max_y_size: 50,
            min_rooms: 1,
            max_rooms: 2,
        }
    }
}

impl DungeonGenerator {
    pub fn generate(&self, settings: &DungeonGenerationSettings) -> Dungeon {
        let num_rooms = (settings.min_rooms..=settings.max_rooms)
            .choose(&mut rand::thread_rng())
            .unwrap();

        let mut rooms = vec![];

        for i in 0..num_rooms {
            rooms.push(self.generate_room(settings));
        }

        Dungeon { rooms }
    }

    fn generate_room(&self, settings: &DungeonGenerationSettings) -> Room {
        let x_size = (settings.min_x_size..=settings.max_x_size)
            .choose(&mut rand::thread_rng())
            .unwrap();

        let y_size = (settings.min_y_size..=settings.max_y_size)
            .choose(&mut rand::thread_rng())
            .unwrap();

        let mut tiles = vec![];

        const TILE_SIZE: u16 = 64;

        let mut transform = Transform::default();

        // Construct our initial random smattering of tiles
        for y in 0..y_size {
            for x in 0..x_size {
                if rand::thread_rng().gen_bool(0.5) {
                    continue;
                }

                transform.translation =
                    Vec3::new((x * TILE_SIZE) as f32, (y * TILE_SIZE) as f32, -10.0);

                tiles.push(Tile {
                    tile_type_id: rand::thread_rng().gen_range(0..=1), /* TODO */
                    transform,
                });
            }
        }

        // Do another pass to ensure we can traverse between all tiles
        // ZJ-TODO

        Room {
            tiles,
            decorations: vec![],
        }
    }

    fn wave_function_collapse(&self) {

    }
}
