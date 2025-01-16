#![allow(dead_code)]

use crate::folder::Folder;
use crate::parser::parse_stmt;
use crate::units::BaseUnits;
use std::error::Error;

pub struct Interpreter {
    pub state: BaseUnits,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Self {
            state: BaseUnits::new(),
        }
    }
}

impl Folder for Interpreter {}

impl Interpreter {
    pub fn convert(&mut self, stmt: &str) -> Result<f64, Box<dyn Error>> {
        // Calculate given conversation

        let stmt = parse_stmt(stmt)?;
        let conv = self.fold_stmt(&stmt)?;

        Ok(conv.v * conv.mpl)
    }

    pub fn decompose(&mut self, stmt: &str) -> Result<BaseUnits, Box<dyn Error>> {
        // Decompose(or simplify :) given unit-expression
        // plz note(!) - only one or zero '/' allowed in input string

        let stmt = parse_stmt(stmt)?;
        let deco = self.fold_stmt(&stmt)?;
        self.state = deco.clone();
        Ok(deco)
    }
}
