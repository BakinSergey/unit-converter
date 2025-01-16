#![allow(unused_imports, dead_code)]
#![allow(clippy::zero_prefixed_literal)]

use crate::units::Unit;
use pathbuf::pathbuf;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub const ACCURACY: usize = 3;

pub fn load_units_from_file(path: PathBuf) -> Result<Vec<Unit>, Box<dyn Error>> {
    let units = {
        let res = fs::read_to_string(path).expect("Can't read file");
        serde_json::from_str(&res)?
    };

    Ok(units)
}

pub fn units() -> HashMap<String, Unit> {
    let c_units = pathbuf!["./voc/c_units.json"];
    let d_units = pathbuf!["./voc/d_units.json"];
    let u_units = pathbuf!["./voc/u_units.json"];

    let cu = load_units_from_file(c_units).unwrap();
    let du = load_units_from_file(d_units).unwrap();
    let uu = load_units_from_file(u_units).unwrap();

    let all_units = vec![cu, du, uu]
        .into_iter()
        .flatten()
        .collect::<Vec<Unit>>();

    let mut units = HashMap::new();

    for unit in all_units {
        units.insert(unit.tag.clone(), unit);
    }

    units
}

pub fn all_units() -> String {
    units().keys().map(|s| &**s).collect::<Vec<_>>().join(", ")
}

pub fn all_prefixes() -> String {
    prefixes()
        .keys()
        .map(|s| &**s)
        .collect::<Vec<_>>()
        .join(", ")
}

//@fmt:off
pub fn prefixes() -> HashMap<&'static str, i32> {
    HashMap::from([
        ("Т", 12i32),
        ("Г", 09i32),
        ("М", 06i32),
        ("к", 03i32),
        ("г", 02i32),
        ("д", -01i32),
        ("с", -02i32),
        ("м", -03i32),
        ("мк", -06i32),
        ("н", -09i32),
        ("п", -12i32),
    ])
}
//@fmt:on
