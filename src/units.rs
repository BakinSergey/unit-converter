use crate::common::prefixes;
use crate::folder::UnitsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::MutexGuard;
use crate::register;
use crate::register::units;

fn f64_one() -> f64 {
    1.0
}
fn i8_one() -> i8 {
    1
}
fn vec_unit_empty() -> Vec<Unit> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct ParsedUnit {
    // DTO for parsed data
    pub pfx: Option<String>,
    pub tag: String,
    pub pow: i8,
    pub den: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub tag: String, // акроним
    #[serde(default = "f64_one")]
    pub mpl: f64, // мультипликатор
    #[serde(default = "i8_one")]
    pub(crate) pow: i8, // экспонента
    #[serde(default = "vec_unit_empty")]
    pub base: Vec<Unit>,
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
}

impl Default for BaseUnits {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseUnits {
    pub fn new() -> BaseUnits {
        Self {
            v: 0.0,
            units: HashMap::new(),
            mpl: 1.0,
        }
    }

    pub fn as_readable(&self) -> String {
        log_data(&Vec::from_iter(self.units.values().cloned()))
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

    fn merge_base_one(&mut self, bu: Unit) {
        match self.units.get_mut(&bu.tag) {
            Some(unit) => {
                unit.mpl *= bu.mpl; // must be 1.0 for all(?)
                unit.pow += bu.pow;
            }
            None => {
                self.units.insert(bu.tag.clone(), bu.clone());
            }
        }
    }

    pub fn reduce(mut self) -> Self {
        let mut units: Vec<Unit> = vec![];
        let mut mpl: f64 = 1.0;
        let voc = register::units();

        for u in self.units.values() {
            // x^0 == 1 - no need to reduce
            if u.pow == 0 {
                // сама единица не редуцируется,
                // но её мультипликатор используется: 1м/1с_м = 100 []
                mpl *= u.mpl;
                continue;
            }

            let (m, b_units) = to_bases(u, &voc);
            units.extend(b_units);
            mpl *= m;
        }
        self.units.clear();

        // fill from scratch with totally bases
        self.mpl = mpl;
        for u in units {
            self.merge_base_one(u)
        };

        self.exclude_zero_powered();
        self
    }

    fn exclude_zero_powered(&mut self) {
        let mut to_remove: Vec<String> = vec![];

        for u in self.units.values().cloned() {
            if u.pow == 0 {
                to_remove.push(u.tag);
            }
        }

        for u in to_remove {
            self.units.remove(&u);
        }
    }

    pub fn add_parsed_unit(&mut self, u: ParsedUnit) -> Result<(), UnitsError> {
        // add dto as unit

        let mut all_units: MutexGuard<HashMap<String, Unit>> = units();
        let all_prefixes: HashMap<&'static str, i32> = prefixes();

        match all_units.get_mut(&u.tag) {
            Some(ex_u) => {
                let mut proto = ex_u.clone();
                // println!("{:?}", ex_u);

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

    pub fn is_coherent(&self, other: &BaseUnits) -> bool {
        // coherent(can be converted to each other)
        self.units == other.units
    }
}

pub fn to_bases(u: &Unit, voc: &HashMap<String, Unit>) -> (f64, Vec<Unit>) {
    // u - единица {mpl, tag, pow, base} к-ю надо представить в базовых

    let mut base_units: Vec<Unit> = vec![];
    let mut mpl: f64 = 1.0;

    if u.base.is_empty() {
        mpl *= u.mpl.powi(u.pow as i32);
        return (mpl, vec![u.clone()]);
    }


    base_units.push(u.clone());

    // reduce to totally base
    loop {
        let mut new_base_units: Vec<Unit> = vec![];

        // log_data(&base_units, &mpl);

        for p in &mut base_units {
            if !p.base.is_empty() {
                // apply parent pow to child
                for c in &mut p.base {
                    c.pow *= p.pow;
                    c.mpl = c.mpl.powi(p.pow as i32);
                }
                new_base_units.extend(p.base.clone());
                continue;
            }

            // get from voc
            if let Some(voc_unit) = voc.get(&p.tag) {
                let mut unit = voc_unit.clone();

                // apply parent pow to child
                for c in &mut unit.base {
                    c.pow *= p.pow;
                    c.mpl = c.mpl.powi(p.pow as i32);
                }
                new_base_units.extend(unit.base);
            }
            mpl *= p.mpl;
        };

        if !new_base_units.is_empty() {
            base_units = new_base_units;
        } else { break; }
    }

    mpl *= u.mpl;
    (mpl, base_units)
}


pub fn log_data(u: &[Unit]) -> String {
    let s = u.iter().map(|s| format!("{}^{}", s.tag, s.pow))
        .collect::<Vec<_>>().join(" * ");
    // format!("{} [{}]", m, s)
    format!("[{}]", s)
}
