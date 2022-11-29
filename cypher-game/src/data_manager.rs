use std::sync::{Arc, Mutex};

use bevy::prelude::Resource;
use cypher_core::{
    affix::database::AffixDefinitionDatabase, affix_pool::database::AffixPoolDefinitionDatabase,
    data::DataDefinitionDatabase,
};
use cypher_item::{
    item::database::ItemDefinitionDatabase, loot_pool::database::LootPoolDefinitionDatabase,
};

#[derive(Resource)]
pub struct DataManager {
    pub affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
    pub affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
    pub item_db: Arc<Mutex<ItemDefinitionDatabase>>,
    pub loot_pool_db: Arc<Mutex<LootPoolDefinitionDatabase>>,
}

impl Default for DataManager {
    fn default() -> DataManager {
        let mut affix_db_path = std::env::current_dir().unwrap();
        affix_db_path.push("cypher-game");
        affix_db_path.push("assets");
        affix_db_path.push("game_data");
        affix_db_path.push("affix.json");
        let affix_db = Arc::new(Mutex::new(AffixDefinitionDatabase::load_from(
            affix_db_path.to_str().unwrap(),
            &(),
        )));

        let mut affix_pool_db_path = std::env::current_dir().unwrap();
        affix_pool_db_path.push("cypher-game");
        affix_pool_db_path.push("assets");
        affix_pool_db_path.push("game_data");
        affix_pool_db_path.push("affix_pool.json");
        let affix_pool_db = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::load_from(
            affix_pool_db_path.to_str().unwrap(),
            &affix_db,
        )));

        let mut item_db_path = std::env::current_dir().unwrap();
        item_db_path.push("cypher-game");
        item_db_path.push("assets");
        item_db_path.push("game_data");
        item_db_path.push("item.json");
        let item_db = Arc::new(Mutex::new(ItemDefinitionDatabase::load_from(
            item_db_path.to_str().unwrap(),
            &affix_pool_db,
        )));

        let mut loot_pool_db_path = std::env::current_dir().unwrap();
        loot_pool_db_path.push("cypher-game");
        loot_pool_db_path.push("assets");
        loot_pool_db_path.push("game_data");
        loot_pool_db_path.push("loot_pool.json");
        let loot_pool_db = Arc::new(Mutex::new(LootPoolDefinitionDatabase::load_from(
            loot_pool_db_path.to_str().unwrap(),
            &item_db,
        )));

        DataManager {
            affix_db,
            affix_pool_db,
            item_db,
            loot_pool_db,
        }
    }
}
