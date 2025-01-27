use crate::register::units;
use crate::register;

pub const ACCURACY: usize = 10;

pub fn all_units() -> String {
    units().keys().map(|s| &**s).collect::<Vec<_>>().join(", ")
}

pub fn all_prefixes() -> String {
    register::prefixes()
        .keys()
        .map(|s| &**s)
        .collect::<Vec<_>>()
        .join(", ")
}
