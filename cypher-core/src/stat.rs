use std::{collections::HashMap, fmt::Display, ops::Add};

use serde::{Deserialize, Serialize};

// Stats are any numeric value a player can possess.
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Eq, Serialize)]
pub enum Stat {
    Resolve,
    Finesse,
    Complexity,
    MoveSpeed,
    Health,
    Energy,
}

#[derive(Clone, PartialEq)]
pub struct StatModifier(pub Stat, pub f32);

impl Display for StatModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {}{}",
            self.0,
            if self.1 >= 0. { "+" } else { "" },
            self.1
        )
    }
}

#[derive(Debug)]
pub struct StatList {
    modifiers: HashMap<Stat, f32>,
}

impl StatList {
    /// Creates a [StatList] from a slice of [StatModifier]s.
    pub fn from(mods: &[StatModifier]) -> StatList {
        let mut stat_list = StatList {
            modifiers: HashMap::new(),
        };

        for modifier in mods {
            stat_list.add_mod(modifier);
        }

        stat_list
    }

    /// Adds a [StatModifier] to this [StatList].
    fn add_mod(&mut self, modifier: &StatModifier) -> &mut StatList {
        match self.modifiers.get_mut(&modifier.0) {
            Some(val) => {
                *val += modifier.1;

                // If the stat would be 0 after modifying, remove it from the list
                if *val == 0. {
                    self.modifiers.remove(&modifier.0);
                }
            }
            None => {
                self.modifiers.insert(modifier.0, modifier.1);
            }
        };

        self
    }

    /// Retrieves all [StatModifier]s provided by the [StatList].
    pub fn mods(&self) -> Vec<StatModifier> {
        self.modifiers
            .iter()
            .map(|modifier| StatModifier(*modifier.0, *modifier.1))
            .collect()
    }

    /// Gets the modifier value of a [Stat] in a [StatList], if it exists.
    pub fn get_stat(&self, stat: &Stat) -> Option<&f32> {
        self.modifiers.get(stat)
    }
}

impl Add for StatList {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        for modifier in rhs.mods() {
            self.add_mod(&modifier);
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_different_stat_modifiers() {
        let resolve_plus_one = StatModifier(Stat::Resolve, 1.);
        let finesse_plus_two = StatModifier(Stat::Finesse, 2.);

        let stat_list = StatList::from(&[resolve_plus_one.clone(), finesse_plus_two.clone()]);
        let mods = stat_list.mods();

        assert_eq!(mods.len(), 2);
        assert!(mods.contains(&resolve_plus_one));
        assert!(mods.contains(&finesse_plus_two));
    }

    #[test]
    fn add_same_stat_modifiers() {
        let resolve_plus_one = StatModifier(Stat::Resolve, 1.);
        let resolve_plus_two = StatModifier(Stat::Resolve, 2.);

        let stat_list = StatList::from(&[resolve_plus_one.clone(), resolve_plus_two.clone()]);
        let mods = stat_list.mods();

        assert_eq!(mods.len(), 1);
        let resolve_plus_three = StatModifier(Stat::Resolve, 3.);
        assert!(mods.contains(&resolve_plus_three));
    }

    #[test]
    fn zero_stat_modifiers_omitted() {
        let resolve_plus_one = StatModifier(Stat::Resolve, 1.);
        let resolve_minus_one = StatModifier(Stat::Resolve, -1.);

        let stat_list = StatList::from(&[resolve_plus_one.clone(), resolve_minus_one.clone()]);
        let mods = stat_list.mods();
        assert_eq!(mods.len(), 0);
    }

    #[test]
    fn add_stat_lists_different_modifiers() {
        let stat_list_1 = StatList::from(&[StatModifier(Stat::Resolve, 1.)]);
        let stat_list_2 = StatList::from(&[StatModifier(Stat::Finesse, 1.)]);

        let new_list = stat_list_1 + stat_list_2;

        assert_eq!(new_list.mods().len(), 2);
        let resolve_plus_one = StatModifier(Stat::Resolve, 1.);
        let finesse_plus_one = StatModifier(Stat::Finesse, 1.);
        assert!(new_list.mods().contains(&resolve_plus_one));
        assert!(new_list.mods().contains(&finesse_plus_one));
    }

    #[test]
    fn add_stat_lists_same_modifiers() {
        let stat_list_1 = StatList::from(&[StatModifier(Stat::Resolve, 1.)]);
        let stat_list_2 = StatList::from(&[StatModifier(Stat::Resolve, 1.)]);

        let new_list = stat_list_1 + stat_list_2;

        assert_eq!(new_list.mods().len(), 1);
        let resolve_plus_two = StatModifier(Stat::Resolve, 2.);
        assert!(new_list.mods().contains(&resolve_plus_two));
    }

    #[test]
    fn get_stat_in_stat_list() {
        let stat_list = StatList::from(&[
            StatModifier(Stat::Resolve, 1.),
            StatModifier(Stat::Finesse, 2.),
        ]);

        let resolve_stat = stat_list.get_stat(&Stat::Resolve);
        let finesse_stat = stat_list.get_stat(&Stat::Finesse);
        let complexity_stat = stat_list.get_stat(&Stat::Complexity);

        assert!(resolve_stat.is_some());
        assert_eq!(*resolve_stat.unwrap(), 1.);

        assert!(finesse_stat.is_some());
        assert_eq!(*finesse_stat.unwrap(), 2.);

        assert!(complexity_stat.is_none());
    }
}
