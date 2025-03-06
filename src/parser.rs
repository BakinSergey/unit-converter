use crate::ast::*;

//@fmt:off
const CONV: &str = "=>";
const MUL: &str  = "*";
const DIV: &str  = "/";
const WS: &str   = " ";
const US: &str   = "_";
const PW: &str   = "^";
//@fmt:on

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("input cannot contain: {0} spaces, 0 or 1 is allowed")]
    TooMuchSpaces(usize),

    #[error("Unit entry is wrong")]
    WrongUnit(String),

    #[error("float input wrong: {0}")]
    ValueWrongBegin(String),

    #[error("Convert Stmt allowed exactly one '=>' occurrence")]
    ExactlyOneExprSeparator(),

    #[error("unknown unit prefix: {0}")]
    UnknownPrefix(String),

    #[error("unknown unit: {0}")]
    UnknownUnit(String),

    #[error("pow cannot be parsed as valid i8: {0}")]
    WrongPow(String),
}

pub fn enter_validation(input: &str) -> Result<&str, ParseError> {
    // So, about our little dsl grammar...
    // valid inputs are:
    //  - [float][WS][unit_expr][CONV][unit_expr]  ( with only one whitespace )
    //  - [unit_expr]
    // unit_expr can contain one or several Unit, separated by many * and|or exactly one /
    // Unit is constrained as: [pfx][US][tag][PW][pow]

    // 0 or 1 space is allowed
    let ws: usize = input.matches(WS).count();
    if ws > 1 {
        return Err(ParseError::TooMuchSpaces(ws));
    }

    // Suppose Conversation operation,
    if ws == 1 {
        let mut inp = input.splitn(2, WS);
        // try get float value
        match inp.next().unwrap().parse::<f64>() {
            Ok(_) => (),
            Err(_) => return Err(ParseError::ValueWrongBegin(input.to_owned())),
        };
        // exactly one '=>' allowed
        let conv = inp.next().unwrap().matches(CONV).count();
        match conv {
            1 => (),
            _ => return Err(ParseError::ExactlyOneExprSeparator()),
        }
    }

    Ok(input)
}

pub fn parse_unit(input: &str, den: bool) -> Result<Expr, ParseError> {
    // parse unit as pfx_tag^pow, where pfx is Option
    // den - is denominator flag

    let mut pfx = None;
    let mut tag = String::new();
    let mut pow = 1;

    if input.is_empty() {
        return Err(ParseError::WrongUnit(input.to_owned()));
    };

    let uc = input.matches(US).count();
    let pw = input.matches(PW).count();

    if uc == 0 && pw == 0 {
        tag = input.to_string();
    }

    if uc == 1 && pw == 0 {
        let mut pt = input.split(US);
        pfx = pt.next().map(|s| s.to_string());
        tag = pt.next().unwrap().to_string();
    }

    if uc == 0 && pw == 1 {
        let mut pt = input.split(PW);
        tag = pt.next().unwrap().to_string();

        if let Some(s) = pt.next() {
            pow = s.parse().unwrap();
        }
    }

    if uc == 1 && pw == 1 {
        let mut pt = input.split(US);
        pfx = pt.next().map(|s| s.to_string());
        let tag_and_pow = pt.next().unwrap().to_string();

        let mut pt = tag_and_pow.rsplit(PW);
        if let Some(s) = pt.next() {
            pow = s.parse().unwrap();
        }
        tag = pt.next().unwrap().to_string();
    }
    Ok(Expr::Unit { pfx, tag, pow, den })
}

pub fn parse_expr(input: &str) -> Result<Expr, ParseError> {
    // Fraction expression, with * and maybe with / (only one / is allowed)
    // all units will be consumed here. if ever.
    // single unit is expr too.

    let mut inp_div_iter = input.split(DIV);

    let mut frac_up: Vec<Expr> = Vec::new();
    let mut frac_dn: Vec<Expr> = Vec::new();

    if let Some(up_expr) = inp_div_iter.next() {
        for e in up_expr.split(MUL) {
            frac_up.push(parse_unit(e, false)?);
        }
    }
    if let Some(dn_expr) = inp_div_iter.next() {
        for e in dn_expr.split(MUL) {
            frac_dn.push(parse_unit(e, true)?);
        }
    }
    Ok(Expr::Fraction {
        up: frac_up,
        down: frac_dn,
    })
}

pub fn parse_stmt(input: &str) -> Result<Stmt, ParseError> {
    // Conversation statement
    if input.contains(CONV) {
        let mut inp = input.split(WS);
        let val: f64 = inp.next().unwrap().parse().unwrap(); // already validated

        let (lft, rht) = inp.next().unwrap().split_once(CONV).unwrap();

        return Ok(Stmt::Conversation(Expr::Convert(
            val,
            Box::new(parse_expr(lft)?),
            Box::new(parse_expr(rht)?),
        )));
    }

    // Decomposition statement
    Ok(Stmt::Decomposition(parse_expr(input)?))
}
