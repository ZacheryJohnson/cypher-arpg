use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

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

impl DataManager {
    pub fn new(game_data_path: PathBuf) -> Self {
        let mut affix_db_path = game_data_path.clone();
        affix_db_path.push("affix.json");
        let affix_db = Arc::new(Mutex::new(AffixDefinitionDatabase::load_from(
            affix_db_path.to_str().unwrap(),
            &(),
        )));

        let mut affix_pool_db_path = game_data_path.clone();
        affix_pool_db_path.push("affix_pool.json");
        let affix_pool_db = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::load_from(
            affix_pool_db_path.to_str().unwrap(),
            &affix_db,
        )));

        let mut item_db_path = game_data_path.clone();
        item_db_path.push("item.json");
        let item_db = Arc::new(Mutex::new(ItemDefinitionDatabase::load_from(
            item_db_path.to_str().unwrap(),
            &(affix_db.clone(), affix_pool_db.clone()),
        )));

        let mut loot_pool_db_path = game_data_path;
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

impl Default for DataManager {
    fn default() -> DataManager {
        let mut base_path = std::env::current_dir().unwrap();
        base_path.push("cypher-game");
        base_path.push("assets");
        base_path.push("game_data");

        DataManager::new(base_path)
    }
}
