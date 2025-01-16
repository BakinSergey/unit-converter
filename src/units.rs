#![allow(unused_imports)]
#![allow(dead_code)]

use crate::common::{prefixes, units};
use crate::folder::UnitsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

fn f64_one() -> f64 {
    1.0
}
fn i8_one() -> i8 {
    1
}
fn vec_unit_empty() -> Vec<Unit> {
    vec![]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub tag: String, // акроним
    #[serde(default = "f64_one")]
    pub mpl: f64, // мультипликатор
    #[serde(default = "i8_one")]
    pub(crate) pow: i8, // экспонента
    #[serde(default = "vec_unit_empty")]
    base: Vec<Unit>,
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.pow == other.pow && self.base.is_empty() && other.base.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseUnits {
    // result of transformaion to Main C units
    // (base, atomic 'seven liberty units' :)
    // - unprefix,
    // - move to numerator(change pow sign, invert multiplier),
    // - recalc multiplier of unit
    pub v: f64,
    pub units: HashMap<String, Unit>,
    pub mpl: f64,
    pub is_done: bool,
}

pub struct ParsedUnit {
    // DTO for parsed data
    pub pfx: Option<String>,
    pub tag: String,
    pub pow: i8,
    pub den: bool,
}

impl BaseUnits {
    pub fn new() -> BaseUnits {
        Self {
            v: 1.0,
            units: HashMap::new(),
            mpl: 1.0,
            is_done: false,
        }
    }

    pub fn as_readable(&self) -> String {
        String::new()
    }

    fn calc_multiplier(&mut self) -> f64 {
        self.mpl = 1.0;
        self.is_done = true;
        for (k, v) in &self.units {
            self.mpl *= v.mpl
        }

        self.mpl
    }

    pub fn merge_one(&mut self, bu: BaseUnits, tag: String) {
        let u = bu.units.get(&tag).unwrap();

        match self.units.get_mut(&tag) {
            Some(unit) => {
                unit.mpl *= u.mpl;
                unit.pow += u.pow;
            }
            None => {
                self.units.insert(tag.clone(), u.clone());
            }
        }
    }

    pub fn add_parsed_unit(&mut self, u: ParsedUnit) -> Result<(), UnitsError> {
        // add dto as unit
        self.is_done = false;
        // so ugly, todo!(better)
        let mut all_units: HashMap<String, Unit> = units();
        let all_prefixes: HashMap<&'static str, i32> = prefixes();

        match all_units.get_mut(&u.tag) {
            Some(ex_u) => {
                let mut proto = ex_u.clone();

                let mut mpl = 1.0f64;
                let mut pow = u.pow;

                // если есть приставка - считаем мультипликатор
                if u.pfx.is_some() {
                    let pfx = u.pfx.unwrap();
                    mpl = match all_prefixes.get(pfx.as_str()) {
                        Some(pfx_mpl) => 10f64.powi(*pfx_mpl * (pow as i32)),
                        None => return Err(UnitsError::NoUnitPrefix(pfx)),
                    };
                }
                // если знаменатель
                if u.den {
                    mpl = 1.0 / mpl;
                    pow = -pow
                };

                proto.mpl = mpl;
                proto.pow = pow;

                match self.units.get_mut(&proto.tag) {
                    Some(ex_u) => {
                        ex_u.mpl *= proto.mpl;
                        ex_u.pow += proto.pow;
                    }
                    None => {
                        self.units.insert(proto.tag.clone(), proto);
                    }
                }
            }
            None => return Err(UnitsError::NoUnit(u.tag.clone())),
        }
        Ok(())
    }

    pub fn add_unit(&mut self, u: Unit) {
        // add 'domain'-unit
        self.is_done = false;
        self.units.insert(u.tag.clone(), u);
    }

    pub fn multiplier(&mut self) -> f64 {
        // calc and set total multiplier of self units
        if self.is_done {
            self.mpl
        } else {
            self.calc_multiplier()
        }
    }

    pub fn is_coherent(&self, other: &BaseUnits) -> bool {
        // coherent(can be converted to each other)
        self.units == other.units
    }
}
