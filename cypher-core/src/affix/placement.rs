use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Serialize)]
pub enum AffixPlacement {
    Invalid,
    Prefix,
    Suffix,
}

impl std::fmt::Display for AffixPlacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
