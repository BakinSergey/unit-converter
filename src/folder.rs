// Folder: transform parsed units to C-System of units

use crate::ast::*;
use crate::units::{BaseUnits, ParsedUnit};

#[derive(Debug, thiserror::Error)]
pub enum UnitsError {
    #[error("units not coherent:{0} <=> {1}")]
    NotCoherent(String, String),

    #[error("unit {0} not found")]
    NoUnit(String),

    #[error("unit prefix {0} not found")]
    NoUnitPrefix(String),
}

pub(crate) trait Folder {
    fn fold_stmt(&mut self, s: &Stmt) -> Result<BaseUnits, UnitsError> {
        let folded = match s {
            Stmt::Conversation(conv) => self.fold_expr(conv)?,
            Stmt::Decomposition(expr) => self.fold_expr(expr)?,
        };
        Ok(folded)
    }

    fn fold_expr(&mut self, e: &Expr) -> Result<BaseUnits, UnitsError> {
        match e {
            Expr::Convert(v, src, dst) => {
                let mut base = BaseUnits::new();
                base.v = *v;

                let src_base = self.fold_expr(src)?;
                let dst_base = self.fold_expr(dst)?;

                // coherent
                if src_base.is_coherent(&dst_base) {
                    base.units = src_base.units;
                    base.mpl = src_base.mpl / dst_base.mpl;

                // not coherent
                } else {
                    return Err(UnitsError::NotCoherent(
                        src_base.as_readable(),
                        dst_base.as_readable(),
                    ));
                };
                Ok(base)
            }

            Expr::Fraction { up, down } => {
                let mut base = BaseUnits::new();

                for unit in up.iter().chain(down.iter()) {
                    match unit {
                        Expr::Unit {
                            pfx: _,
                            tag: t,
                            pow: _,
                            den: _,
                        } => {
                            let folded = self.fold_expr(unit)?;
                            base.merge_one(folded, t.to_string());
                        }
                        _ => return Err(UnitsError::NoUnit("sorry".into())),
                    }
                }
                // reduce to bases
                // до вызова reduce, bases содержит единицы после парсинга,
                // т.е. не приведенные к самым базовым
                // (но уже в формате базовых единиц(pfx -> mpl)).
                // reduce - приводит все единицы к самым базовым.

                // mpl of base here is 1.0
                base = base.reduce();
                // mpl of base here != 1.0
                Ok(base)
            }

            Expr::Unit {
                pfx: p,
                tag: t,
                pow: w,
                den: d,
            } => {
                // ParsedUnit -> Unit
                let mut base = BaseUnits::new();
                let p_unit = ParsedUnit {
                    pfx: p.clone(),
                    tag: t.clone(),
                    pow: *w,
                    den: *d,
                };
                base.add_parsed_unit(p_unit)?;
                Ok(base)
            }
        }
    }
}
