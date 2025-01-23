use crate::common::load_units;
use crate::units::Unit;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

pub static UNITS: OnceCell<Mutex<HashMap<String, Unit>>> = OnceCell::new();

pub fn init_units() {
    let units: HashMap<String, Unit> = load_units();
    UNITS.get_or_init(|| Mutex::new(units));
}

pub fn add_unit(u: Unit) {
    UNITS.get().unwrap().lock().unwrap().insert(u.tag.clone(), u);
}

pub fn units() -> MutexGuard<'static, HashMap<String, Unit>> {
    UNITS.get().unwrap().lock().unwrap()
}