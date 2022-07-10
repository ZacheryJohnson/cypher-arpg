use cypher_core::stat::{Stat, StatList};

pub struct Character {
    stats: Vec<StatList>,

    // TODO: refactor
    current_health: u32,
}

impl Character {
    pub fn new(stat_lists: Vec<StatList>) -> Character {
        let mut new_char = Character {
            stats: stat_lists,
            current_health: 0,
        };

        new_char.current_health = new_char
            .stats
            .iter()
            .map(|stat_list| *stat_list.get_stat(&Stat::Health).unwrap_or(&0.))
            .reduce(|acc, next| acc + next)
            .unwrap()
            .floor() as u32;

        new_char
    }

    /// Combines all player [StatList]s into a singular, cumulative [StatList].
    /// **This isn't performant**: creates a temporary. Should instead be implemented with iterators.
    pub fn stats(&self) -> StatList {
        let mut new_list = StatList::from(&[]);
        for stat_list in &self.stats {
            new_list.add_list(stat_list);
        }

        new_list
    }
}

#[cfg(test)]
mod tests {
    use cypher_core::stat::{Stat, StatList, StatModifier};

    use super::*;

    #[test]
    fn new_sets_current_health_to_max() {
        let stat_list_1 = StatList::from(&[StatModifier(Stat::Health, 3.)]);
        let stat_list_2 = StatList::from(&[StatModifier(Stat::Health, 5.8)]);

        let character = Character::new(vec![stat_list_1, stat_list_2]);
        assert_eq!(character.current_health, 8);
    }

    #[test]
    fn can_combine_stat_lists() {
        let stat_list_1 = StatList::from(&[StatModifier(Stat::Resolve, 1.)]);
        let stat_list_2 = StatList::from(&[StatModifier(Stat::Resolve, 1.)]);

        let character = Character::new(vec![stat_list_1, stat_list_2]);

        let combined = character.stats();

        assert_eq!(*combined.get_stat(&Stat::Resolve).unwrap(), 2.);
    }
}
