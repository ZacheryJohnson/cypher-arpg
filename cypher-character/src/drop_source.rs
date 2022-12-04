use cypher_item::loot_pool::definition::LootPoolDefinition;

pub trait DropSource {
    fn loot_pool(&self) -> &LootPoolDefinition;
}
