use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::types::ids::VariableId;
use crate::types::monomial::{monomial_degree, Monomial};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonomialOrder {
    Lex(Vec<VariableId>),
    Grevlex(Vec<VariableId>),
    Elimination {
        eliminate: Vec<VariableId>,
        keep: Vec<VariableId>,
    },
    Block(Vec<Vec<VariableId>>),
}

impl MonomialOrder {
    pub fn compare(&self, a: &Monomial, b: &Monomial) -> Ordering {
        match self {
            MonomialOrder::Lex(vars) => compare_lex(a, b, vars),
            MonomialOrder::Grevlex(vars) => compare_grevlex(a, b, vars),
            MonomialOrder::Elimination { eliminate, keep } => {
                let mut vars = eliminate.clone();
                vars.extend_from_slice(keep);
                compare_lex(a, b, &vars)
            }
            MonomialOrder::Block(blocks) => {
                for block in blocks {
                    let ord = compare_lex(a, b, block);
                    if ord != Ordering::Equal {
                        return ord;
                    }
                }
                Ordering::Equal
            }
        }
    }
}

pub fn elimination_order(eliminate: &[VariableId], keep: &[VariableId]) -> MonomialOrder {
    MonomialOrder::Elimination {
        eliminate: eliminate.to_vec(),
        keep: keep.to_vec(),
    }
}

pub fn grevlex_order(vars: &[VariableId]) -> MonomialOrder {
    MonomialOrder::Grevlex(vars.to_vec())
}

pub fn lex_order(vars: &[VariableId]) -> MonomialOrder {
    MonomialOrder::Lex(vars.to_vec())
}

pub fn block_order(blocks: Vec<Vec<VariableId>>) -> MonomialOrder {
    MonomialOrder::Block(blocks)
}

fn exponent(m: &Monomial, v: VariableId) -> u32 {
    m.exponents
        .iter()
        .find(|(var, _)| *var == v)
        .map(|(_, exp)| *exp)
        .unwrap_or(0)
}

fn compare_lex(a: &Monomial, b: &Monomial, vars: &[VariableId]) -> Ordering {
    for var in vars {
        match exponent(a, *var).cmp(&exponent(b, *var)) {
            Ordering::Equal => {}
            ord => return ord,
        }
    }
    a.cmp(b)
}

fn compare_grevlex(a: &Monomial, b: &Monomial, vars: &[VariableId]) -> Ordering {
    match monomial_degree(a).cmp(&monomial_degree(b)) {
        Ordering::Equal => {
            for var in vars.iter().rev() {
                match exponent(b, *var).cmp(&exponent(a, *var)) {
                    Ordering::Equal => {}
                    ord => return ord,
                }
            }
            Ordering::Equal
        }
        ord => ord,
    }
}
