#![allow(unused_variables)]

mod ast;
mod common;
mod folder;
mod interpreter;
mod parser;
mod units;

use common::units;

fn main() {
    let um = units();
    print!("{:#?}", um);
}

#[cfg(test)]
mod test_validation {
    use crate::parser::*;

    #[test]
    fn test_enter_validation() {
        // two spaces
        assert!(enter_validation("1  кг=>т")
            .is_err_and(|e| e.to_string().contains("0 or 1 is allowed")));

        // wrong f32
        assert!(enter_validation("f1 кг=>т")
            .is_err_and(|e| e.to_string().contains("float input wrong")));

        // wrong f32
        assert!(enter_validation("1,0 кг=>т")
            .is_err_and(|e| e.to_string().contains("float input wrong")));

        // exactly 1 =>
        assert!(enter_validation("10.1 кг")
            .is_err_and(|e| e.to_string().contains("one '=>' occurrence")));

        // exactly 1 =>
        assert!(enter_validation("10.1 =>кг=>")
            .is_err_and(|e| e.to_string().contains("one '=>' occurrence")));
    }
}

#[cfg(test)]
mod test_common {
    use crate::ast::Expr;

    pub(crate) fn check_unit_parse(e: Expr, pfx: Option<String>, tag: String, pow: i8, den: bool) {
        // println!("{e:?}");
        match e {
            Expr::Unit {
                pfx: p,
                tag: t,
                pow: w,
                den: d,
            } => {
                assert_eq!(p, pfx);
                assert_eq!(t, tag);
                assert_eq!(w, pow);
                assert_eq!(d, den);
            }
            _ => panic!("test failed"),
        };
    }

    pub(crate) fn check_fraction_parse(e: Expr, up: Vec<Expr>, dn: Vec<Expr>) {
        // println!("{e:?}");
        match e {
            Expr::Fraction {
                up: u_vec,
                down: d_vec,
            } => {
                // numerator
                for (i, u) in u_vec.into_iter().enumerate() {
                    match up.get(i).unwrap() {
                        Expr::Unit {
                            pfx: p,
                            tag: t,
                            pow: w,
                            den: d,
                        } => check_unit_parse(u, p.clone(), t.clone(), *w, *d),
                        _ => (),
                    }
                }
                // denominator
                for (j, u) in d_vec.into_iter().enumerate() {
                    match dn.get(j).unwrap() {
                        Expr::Unit {
                            pfx: p,
                            tag: t,
                            pow: w,
                            den: d,
                        } => check_unit_parse(u, p.clone(), t.clone(), *w, *d),
                        _ => (),
                    }
                }
            }
            _ => panic!("test failed"),
        }
    }

    pub(crate) fn check_convert_parse(
        e: Expr,
        val: f64,
        src_up: Vec<Expr>,
        src_dn: Vec<Expr>,
        dst_up: Vec<Expr>,
        dst_dn: Vec<Expr>,
    ) {
        // println!("{e:?}");
        match e {
            Expr::Convert(v, src_frac, dst_frac) => {
                assert_eq!(v, val);
                // source fraction
                check_fraction_parse(*src_frac, src_up, src_dn);
                // destination fraction
                check_fraction_parse(*dst_frac, dst_up, dst_dn);
            }
            _ => panic!("test failed"),
        }
    }
}

#[cfg(test)]
mod test_parse_unit {
    use crate::parser::parse_unit;
    use crate::test_common::check_unit_parse;

    #[test]
    fn test_parse_unit_nyn() {
        // prefix N; tag Y; pow N;
        let ast = parse_unit("кг", false).unwrap();
        check_unit_parse(ast, None, "кг".to_string(), 1, false);
    }

    #[test]
    fn test_parse_unit_yyn() {
        // prefix Y; tag Y; pow N
        let ast = parse_unit("мк_кг", false).unwrap();
        check_unit_parse(ast, Some("мк".to_string()), "кг".to_string(), 1, false);
    }

    #[test]
    fn test_parse_unit_nyy() {
        // prefix Y; tag Y; pow N
        let ast = parse_unit("кг^3", false).unwrap();
        check_unit_parse(ast, None, "кг".to_string(), 3, false);
    }

    #[test]
    fn test_parse_unit_yyy() {
        // prefix Y; tag Y; pow Y
        let ast = parse_unit("мк_кг^3", false).unwrap();
        check_unit_parse(ast, Some("мк".to_string()), "кг".to_string(), 3, false);
    }
}

#[cfg(test)]
mod test_parse_expr {
    use crate::ast::{Expr, Stmt};
    use crate::parser::{parse_expr, parse_stmt, parse_unit};
    use crate::test_common::{check_convert_parse, check_fraction_parse};

    #[test]
    fn test_parse_fraction_yn() {
        // numerator Y; denominator N;
        let ex = "м_г^3*см^2*к_с^-1";

        let n1: Expr = parse_unit("м_г^3", false).unwrap();
        let n2: Expr = parse_unit("см^2", false).unwrap();
        let n3: Expr = parse_unit("к_с^-1", false).unwrap();

        let up: Vec<Expr> = vec![n1, n2, n3];
        let dn: Vec<Expr> = vec![];

        let frac: Expr = parse_expr(ex).unwrap();
        check_fraction_parse(frac, up, dn);
    }

    #[test]
    fn test_parse_fraction_yy() {
        // numerator Y; denominator Y;
        let ex = "м_г^3*см^2*к_с^-1/Т_Гц^2*д_м^3";

        let n1: Expr = parse_unit("м_г^3", false).unwrap();
        let n2: Expr = parse_unit("см^2", false).unwrap();
        let n3: Expr = parse_unit("к_с^-1", false).unwrap();

        let d1: Expr = parse_unit("Т_Гц^2", true).unwrap();
        let d2: Expr = parse_unit("д_м^3", true).unwrap();

        let up: Vec<Expr> = vec![n1, n2, n3];
        let dn: Vec<Expr> = vec![d1, d2];

        let frac: Expr = parse_expr(ex).unwrap();
        check_fraction_parse(frac, up, dn);
    }

    #[test]
    fn test_parse_convert_yy_yn() {
        // src: numerator Y; denominator Y,
        // dst: numerator Y; denominator N;
        let ex = "11.3 м_г^3*см^2*к_с^-1/Т_Гц^2*д_м^3=>н_Па^-2";

        let vl: f64 = 11.3;

        let sn1: Expr = parse_unit("м_г^3", false).unwrap();
        let sn2: Expr = parse_unit("см^2", false).unwrap();
        let sn3: Expr = parse_unit("к_с^-1", false).unwrap();

        let sd1: Expr = parse_unit("Т_Гц^2", true).unwrap();
        let sd2: Expr = parse_unit("д_м^3", true).unwrap();

        let src_up: Vec<Expr> = vec![sn1, sn2, sn3];
        let src_dn: Vec<Expr> = vec![sd1, sd2];

        let dn1: Expr = parse_unit("н_Па^-2", false).unwrap();

        let dst_up: Vec<Expr> = vec![dn1];
        let dst_dn: Vec<Expr> = vec![];
        let convert: Stmt = parse_stmt(ex).unwrap();

        match convert {
            Stmt::Conversation(conv) => {
                check_convert_parse(conv, vl, src_up, src_dn, dst_up, dst_dn);
            }
            _ => panic!("test failed"),
        }
    }
}

#[cfg(test)]
mod test_parse_stmt {
    use crate::ast::Stmt;
    use crate::parser::parse_stmt;

    #[test]
    fn test_statement_dispatch() {
        // if input contain =>  stmt will be Conversation
        let ex = "11.3 м_г^3*см^2*к_с^-1/Т_Гц^2*д_м^3=>н_Па^-2";

        let stmt: Stmt = parse_stmt(ex).unwrap();
        println!("{stmt:?}");
        match stmt {
            Stmt::Conversation(ast) => {}
            _ => panic!("test failed"),
        }

        // else stmt will be Decomposition
        let ex = "м_г^3*см^2*к_с^-1/Т_Гц^2*д_м^3*н_Па^-2";

        let stmt: Stmt = parse_stmt(ex).unwrap();
        println!("{stmt:?}");
        match stmt {
            Stmt::Decomposition(ast) => {}
            _ => panic!("test failed"),
        }

        // ...and simple unit will be Decomposition too
        let ex = "н_Па^-2";

        let stmt: Stmt = parse_stmt(ex).unwrap();
        println!("{stmt:?}");
        match stmt {
            Stmt::Decomposition(ast) => {}
            _ => panic!("test failed"),
        }
    }
}

#[cfg(test)]
mod test_parse_folder {
    use crate::common::ACCURACY;
    use crate::interpreter::Interpreter;

    #[test]
    fn test_fold_convert_1() {
        let exp = "1.12e3 к_м^2*д_м*н_м^3/м_м^4=>п_м^2";

        let mut interpreter = Interpreter::new();
        let res = interpreter.convert(&exp).unwrap();
        let mut parts = exp.split("=>");

        let src = parts.next().unwrap();
        let dst = parts.next().unwrap();

        let res = if res < 1e3 {
            format!("{:.*}", ACCURACY, res)
        } else {
            format!("{:e}", res)
        };

        let output = format!("{} = {} {}", src, res, dst);
        println!("{:#?}", output);
    }

    #[test]
    fn test_fold_decompose_1() {
        let exp = "мк_г^3*см^2*к_с^-1/Т_Гц^2*д_м^3*н_Па^-2";
        let mut interpreter = Interpreter::new();

        interpreter.decompose(&exp).unwrap();
        println!("{:#?}", interpreter.state);
    }

    #[test]
    fn test_fold_decompose_2() {
        // все единицы будут сведены к метру в степени -1
        // 3+2-1-2-3 = -1
        let exp = "мк_м^3*с_м^2*к_м^-1/н_м^2*д_м^3";
        let mut interpreter = Interpreter::new();

        interpreter.decompose(&exp).unwrap();
        println!("{:#?}", interpreter.state);

        assert_eq!(interpreter.state.units.len(), 1);
        assert_eq!(interpreter.state.units.get("м").unwrap().pow, -1);
    }
}
