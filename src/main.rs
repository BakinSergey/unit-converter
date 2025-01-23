#![allow(unused_variables)]

pub mod ast;
pub mod common;
pub mod folder;
pub mod interpreter;
pub mod parser;
pub mod units;
pub mod register;

use crate::common::all_units;
use crate::interpreter::Interpreter;
use crate::register::init_units;

fn main() {

    let deco = true;
    let deco = false;

    let exp = "Па/дм^2";
    // let exp = "1 акр^2=>м^4";
    let exp = "1 акр^2/сут^3=>м^4/с^3";

    init_units();
    println!("{}", all_units());

    let mut ii = Interpreter::new();
    if deco {
        println!("{:#?}", ii.deco(exp).unwrap());
    } else {
        println!("{:#?}", ii.conv(exp).unwrap());
    }
}

#[cfg(test)]
mod test_common {
    use crate::ast::Expr;
    pub use crate::register::init_units;

    pub const EPS: f64 = 0.001;

    #[allow(dead_code)]
    pub fn test_conv_data<'a>() -> Vec<(u8, &'a str, f64)> {
        vec![
            (1, "1 Па^2=>Н^2/м^4", 1.0),
            (2, "1 Па=>Н/м^2", 1.0),
            (3, "1 к_Па=>Н/м^2", 1000.0),
            (4, "1 к_Па^2=>Н^2/м^4", 1000000.0),
            (5, "1 к_Па^2/сут^3=>кг^2/м^2*с^7", 1.55045359574252e-9),
            (6, "1 к_Па^2/м_сут^3=>кг^2/м^2*с^7", 1.55045359574252),
            (7, "1 к_Па^2/см^3=>кг^2/м^5*с^4", 1e12),
            // (8, "1 к_Па^2/с_м^3=>кг^2/м^5*с^4", 1e12), ??? 1e-12
            (9, "1 к_Па^2/км^2=>кг^2/м^4*с^4", 1.0),
            // (9, "1 к_Па^2/к_м^2=>кг^2/м^4*с^4", 1.0),  ??? 1e18
            (10, "1 ч^2=>с^2", 1.296e7),
            (11, "1 сут^2=>с^2", 7.46496e9),
            (12, "1 сут=>с", 86400.0),
            (13, "1 мес30^2=>с^2", 6.94427904e12),
            (14, "1 км/ч=>м/с", 0.2777777778),
            (15, "1 сут^2/кгс^2=>с^6/кг^2*м^2", 77622233.2930381),
            // (16, "1 кг/м^3=>м_г/д_м^3", 1000.0), ??? 1e15
            (16, "1 кг/м^3=>м_г/дм^3", 1000.0),
            // (17, "1 атм/м^2=>Па/д_м^2", 1.01325e3),
            (17, "1 атм/м^2=>Па/дм^2", 1.01325e3),
            (18, "1 атм=>Па", 1.01325e5),
            (19, "1 тс^3/В^2=>кг*А^2*м^-1", 9.431092984355795e11),
            (20, "1 акр^2=>м^4", 16377075.8596),
            (21, "1 кгс=>кг*м/с^2", 9.80665),
            (22, "1 кгс^3=>кг^3*м^3/с^6", 943.1092984356),
            (23, "1 сут^2/кгс^2=>с^6/кг^2*м^2", 77622233.2930381),
            (24, "1 акр^2/сут^3=>м^4/с^3", 2.5391851259e-8),
            (25, "1 акр^2/сут^2=>м^4/с^2", 2.1938559488e-3),
        ]
    }
    #[allow(dead_code)]
    pub fn test_deco_data<'a>() -> Vec<(u8, &'a str, &'a str)> {
        vec![
            (1, "Па/дм^2", "100.000 [кг^1 * м^-3 * с^-2]"),
        ]
    }

    pub(crate) fn check_unit_parse(e: Expr, pfx: Option<String>, tag: String, pow: i8, den: bool) {
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
        match e {
            Expr::Fraction {
                up: u_vec,
                down: d_vec,
            } => {
                // numerator
                for (i, u) in u_vec.into_iter().enumerate() {
                    if let Expr::Unit {
                        pfx: p,
                        tag: t,
                        pow: w,
                        den: d,
                    } = up.get(i).unwrap() { check_unit_parse(u, p.clone(), t.clone(), *w, *d) }
                }
                // denominator
                for (j, u) in d_vec.into_iter().enumerate() {
                    if let Expr::Unit {
                        pfx: p,
                        tag: t,
                        pow: w,
                        den: d,
                    } = dn.get(j).unwrap() { check_unit_parse(u, p.clone(), t.clone(), *w, *d) }
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
mod test_parser_validation {
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
mod test_parser_unit {
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
mod test_parser_expr {
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
mod test_parser_stmt {
    use crate::ast::Stmt;
    use crate::parser::parse_stmt;

    #[test]
    fn test_statement_dispatch() {
        // if input contain =>  stmt will be Conversation
        let ex = "11.3 м_г^3*см^2*к_с^-1/Т_Гц^2*д_м^3=>н_Па^-2";

        let stmt: Stmt = parse_stmt(ex).unwrap();
        match stmt {
            Stmt::Conversation(_ast) => {}
            _ => panic!("test failed"),
        }

        // else stmt will be Decomposition
        let ex = "м_г^3*см^2*к_с^-1/Т_Гц^2*д_м^3*н_Па^-2";

        let stmt: Stmt = parse_stmt(ex).unwrap();
        match stmt {
            Stmt::Decomposition(_ast) => {}
            _ => panic!("test failed"),
        }

        // ...and simple unit will be Decomposition too
        let ex = "н_Па^-2";

        let stmt: Stmt = parse_stmt(ex).unwrap();
        match stmt {
            Stmt::Decomposition(_ast) => {}
            _ => panic!("test failed"),
        }
    }
}

#[cfg(test)]
mod test_folder {
    use crate::register::units;
    use crate::units::{to_bases, BaseUnits, ParsedUnit, Unit};
    use crate::test_common::init_units;

    fn log(h: &str, src: Unit, m: f64, v: Vec<Unit>) {
        println!("1 {:?} = {:?} {:?}^{:?}", h, src.mpl, src.tag, src.pow);
        println!("{:?} {:?}", m, v);
    }


    #[test]
    fn test_to_bases_fn_synth() {
        init_units();
        let voc = units();

        // A: 2 B^2
        //    4 C^3
        // B: 3 D^1
        // C: 2 E^2
        //    4 F^3
        // D: 7 H^2
        // E: 4 H^1
        // F: 5 H^3
        // H: []

        let mut a: Unit = voc.get("A").unwrap().clone();
        let ba = {
            a.pow = 1;
            a.mpl = 1.0;
            a
        };
        let (m, v) = to_bases(&ba, &voc);
        log("A", ba, m, v.clone());
        println!("========");

        let mut a: Unit = voc.get("A").unwrap().clone();
        let a2 = {
            a.pow = 2;
            a.mpl = 1.0;
            a
        };
        let (m, v) = to_bases(&a2, &voc);
        log("A^2", a2, m, v.clone());
        println!("========");

        let mut a: Unit = voc.get("A").unwrap().clone();
        let ka = {
            a.pow = 1;
            a.mpl = 1e3;
            a
        };
        let (m, v) = to_bases(&ka, &voc);
        log("кA", ka, m, v.clone());
        println!("========");

        let mut a: Unit = voc.get("A").unwrap().clone();
        let sa = {
            a.pow = 1;
            a.mpl = 1e-2;
            a
        };
        let (m, v) = to_bases(&sa, &voc);
        log("sA", sa, m, v.clone());
        println!("========");


        let mut a: Unit = voc.get("A").unwrap().clone();
        let ka2 = {
            a.pow = 2;
            a.mpl = 1e3;
            a
        };
        let (m, v) = to_bases(&ka2, &voc);
        log("кA^2", ka2, m, v.clone());
        println!("========");

        let mut a: Unit = voc.get("A").unwrap().clone();
        let sa2 = {
            a.pow = 2;
            a.mpl = 1e-2;
            a
        };
        let (m, v) = to_bases(&sa2, &voc);
        log("sA^2", sa2, m, v.clone());
        println!("========");
    }

    #[test]
    fn test_to_bases_fn() {
        init_units();
        let voc = units();

        // к_Н
        let mut n: Unit = voc.get("Н").unwrap().clone();
        let kn = {
            n.pow = 1;
            n.mpl = 1e3;
            n
        };
        let (m, v) = to_bases(&kn, &voc);
        log("кН", kn, m, v.clone());
        println!("========");

        // к_Н^2
        let mut n: Unit = voc.get("Н").unwrap().clone();
        let kn2 = {
            n.pow = 2;
            n.mpl = 1e6_f64;
            n
        };
        let (m, v) = to_bases(&kn2, &voc);
        log("кН^2", kn2, m, v.clone());
        println!("========");

        // d_Н^3
        let mut n: Unit = voc.get("Н").unwrap().clone();
        let dn3 = {
            n.pow = 3;
            n.mpl = 1e-3_f64;
            n
        };
        let (m, v) = to_bases(&dn3, &voc);
        log("дН^3", dn3, m, v.clone());
        println!("========");

        let mut n: Unit = voc.get("кгс").unwrap().clone();
        let kgs = {
            n.pow = 1;
            n.mpl = 1.0;
            n
        };
        let (m, v) = to_bases(&kgs, &voc);
        log("кгс", kgs, m, v.clone());
        println!("========");
    }

    #[test]
    fn test_add_parsed_unit_fn() {
        // p1: den:Y, кН^2 -> mpl: 1/10^9 pow:-2
        // p2: den:N, МН^3 -> mpl: 1/10^9 pow:-2
        init_units();

        let mut bu = BaseUnits::default();

        let p1 = ParsedUnit {
            pfx: Some("к".to_string()),
            tag: "Н".to_string(),
            pow: 2,
            den: true,
        };

        let p2 = ParsedUnit {
            pfx: Some("М".to_string()),
            tag: "Н".to_string(),
            pow: 3,
            den: false,
        };

        let p3 = ParsedUnit {
            pfx: Some("г".to_string()),
            tag: "сут".to_string(),
            pow: 2,
            den: false,
        };

        let p1_mpl = 1.0 / 10f64.powi(3).powi(2i32); // к_Н^2
        let p2_mpl = 10f64.powi(6).powi(3i32);       // М_Н^3
        let p3_mpl = 10f64.powi(2).powi(2i32);       // г_сут^2

        let _ = bu.add_parsed_unit(p1.clone());
        assert_eq!(bu.units.get(&p1.tag).unwrap().mpl, p1_mpl);
        assert_eq!(bu.units.get(&p1.tag).unwrap().pow, -p1.pow); // denominator

        let _ = bu.add_parsed_unit(p2.clone());
        assert_eq!(bu.units.get(&p1.tag).unwrap().mpl, p1_mpl * p2_mpl);
        assert_eq!(bu.units.get(&p1.tag).unwrap().pow, -p1.pow + p2.pow);

        let _ = bu.add_parsed_unit(p3.clone());
        assert_eq!(bu.units.get(&p1.tag).unwrap().mpl, p1_mpl * p2_mpl);
        assert_eq!(bu.units.get(&p1.tag).unwrap().pow, -p1.pow + p2.pow);

        assert_eq!(bu.units.get(&p3.tag).unwrap().mpl, p3_mpl);
        assert_eq!(bu.units.get(&p3.tag).unwrap().pow, p3.pow);

        assert_eq!(bu.mpl, 1.0);
    }
}

#[cfg(test)]
mod test_interpreter {
    use crate::interpreter::Interpreter;
    use crate::register::init_units;
    use crate::test_common::{test_conv_data, test_deco_data, EPS};


    #[test]
    fn test_interpreter_deco() {
        // декомпозиции
        init_units();
        let mut ii = Interpreter::new();

        for (i, deco, expected) in test_deco_data().iter() {
            match ii.deco(deco) {
                Ok(v) => {
                    print!("{i:5} ");
                    assert_eq!(v, *expected);
                    println!("PASSED: {deco} = {v}");
                }
                Err(_e) => assert!(false),
            }
        }
    }

    #[test]
    fn test_interpreter_conv() {
        // конверсии
        init_units();
        let mut ii = Interpreter::new();

        for (i, conv, ex_mpl) in test_conv_data().iter() {
            match ii.conv_f64(conv) {
                Ok(v) => {
                    print!("{i:5} ");
                    assert!((v - ex_mpl).abs() < EPS);
                    println!("PASSED: {conv} = {v}");
                }
                Err(_e) => assert!(false),
            }
        }
    }
}
