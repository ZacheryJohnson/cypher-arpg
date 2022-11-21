use cypher_item::loot_pool::LootPoolDefinition;

pub trait DropSource {
    fn loot_pool(&self) -> &LootPoolDefinition;
}
