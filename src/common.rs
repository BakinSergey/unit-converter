use crate::register::units;
use crate::units::Unit;
use pathbuf::pathbuf;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub const ACCURACY: usize = 10;

pub fn load_units_from_file(path: PathBuf) -> Result<Vec<Unit>, Box<dyn Error>> {
    let units = {
        let res = fs::read_to_string(path).expect("Can't read file");
        serde_json::from_str(&res)?
    };

    Ok(units)
}

pub fn load_units() -> HashMap<String, Unit> {
    let c_units = pathbuf!["./voc/c_units.json"];
    let cu = load_units_from_file(c_units).unwrap();

    let d_units = pathbuf!["./voc/d_units.json"];
    let du = load_units_from_file(d_units).unwrap();

    let u_units = pathbuf!["./voc/u_units.json"];
    let uu = load_units_from_file(u_units).unwrap();

    let a_units = pathbuf!["./voc/test_ABC_units.json"];
    let au = load_units_from_file(a_units).unwrap();

    let t_units = pathbuf!["./voc/test_units.json"];
    let tu = load_units_from_file(t_units).unwrap();

    let all_units = vec![cu, du, uu, au, tu]
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
        ("Т",  12),
        ("Г",  9),
        ("М",  6),
        ("к",  3),
        ("г",  2),
        ("да", 1),
        ("д", -1),
        ("с", -2),
        ("м", -3),
        ("мк",-6),
        ("н", -9),
        ("п", -12),
    ])
}
//@fmt:on
