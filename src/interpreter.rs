use crate::folder::Folder;
use crate::parser::{enter_validation, parse_stmt};
use crate::units::{log_data, BaseUnits, Unit};
use std::error::Error;

impl Folder for Interpreter {}

pub struct Interpreter {
    pub state: BaseUnits,
}

impl Interpreter {

    pub fn new() -> Interpreter {
        Self { state: BaseUnits::new() }
    }

    pub fn conv_f64(&mut self, stmt: &str) -> Result<f64, Box<dyn Error>> {
        // Calculate given conversation
        enter_validation(stmt)?;
        let stmt_ast = parse_stmt(stmt)?;
        match self.fold_stmt(&stmt_ast) {
            Ok(conv) => { Ok(conv.v * conv.mpl) },
            Err(err) => { Ok(0.0) }
        }
    }

    pub fn conv(&mut self, stmt: &str) -> Result<String, Box<dyn Error>> {
        let res = self.conv_f64(stmt)?;
        let res = format!("{:e}", res);
        let (l, r) = stmt.split_once("=>").unwrap();
        Ok(format!("{l} => {res} {r}"))
    }

    pub fn deco(&mut self, stmt: &str) -> Result<String, Box<dyn Error>> {
        enter_validation(stmt)?;

        println!("{}", stmt);
        println!();

        let stmt = parse_stmt(stmt)?;
        let deco = self.fold_stmt(&stmt)?;
        self.state = deco.clone();

        let mut  rf: Vec<Unit> = deco.units.values().cloned().
            into_iter().map(|x| x).collect::<Vec<Unit>>();
        let mpl = format!("{:.3}", self.state.mpl);

        rf.sort_by(|a, b| {
            a.tag.cmp(&b.tag)
                .then(a.pow.cmp(&b.pow))
                .then(a.mpl.partial_cmp(&b.mpl).unwrap())
        });

        let res = format!("{} {}", mpl, log_data(&rf));
        Ok(res)
    }
}
