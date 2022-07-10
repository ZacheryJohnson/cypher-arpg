use cypher_core::stat::StatList;

pub struct Character {
    stats: Vec<StatList>,
}

impl Character {
    pub fn stats(&self) -> StatList {
        let mut new_list = StatList::from(&[]);
        for stat_list in &self.stats {
            new_list.add_list(stat_list);
        }

        new_list
    }
}
