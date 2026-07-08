use crate::{AlgebraicRealRoot, TargetCertificate, UniPolynomialQ};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactTargetImageCertificate {
    pub cover: TargetCertificate,
    pub squarefree_support: UniPolynomialQ,
    pub root_classifications: Vec<RealRootFiberCertificate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertifiedExactTargetImage {
    pub support: UniPolynomialQ,
    pub squarefree_support: UniPolynomialQ,
    pub values: Vec<AlgebraicRealRoot>,
    pub rejected_roots: Vec<AlgebraicRealRoot>,
    pub certificate: ExactTargetImageCertificate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RealRootFiberCertificate {
    Nonempty {
        root: AlgebraicRealRoot,
        certificate: RealFiberNonemptyCertificate,
    },
    Empty {
        root: AlgebraicRealRoot,
        certificate: RealFiberEmptyCertificate,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealFiberNonemptyCertificate {
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RealFiberEmptyCertificate {
    pub description: String,
}

#[derive(Clone, Debug)]
pub(crate) enum ExactImageClassification {
    Complete(CertifiedExactTargetImage),
    Incomplete {
        unclassified_roots: Vec<AlgebraicRealRoot>,
    },
}

pub(crate) fn classify_real_fibers_conservative(
    cover: TargetCertificate,
    support: UniPolynomialQ,
    squarefree_support: UniPolynomialQ,
    roots: Vec<AlgebraicRealRoot>,
) -> ExactImageClassification {
    let _ = (cover, support, squarefree_support);
    ExactImageClassification::Incomplete {
        unclassified_roots: roots,
    }
}

pub(crate) fn build_certified_exact_target_image(
    cover: TargetCertificate,
    support: UniPolynomialQ,
    squarefree_support: UniPolynomialQ,
    roots: &[AlgebraicRealRoot],
    root_classifications: Vec<RealRootFiberCertificate>,
) -> Option<CertifiedExactTargetImage> {
    if root_classifications.len() != roots.len() {
        return None;
    }

    let mut values = Vec::new();
    let mut rejected_roots = Vec::new();
    for root in roots {
        let Some(classification) = root_classifications
            .iter()
            .find(|classification| classified_root(classification) == root)
        else {
            return None;
        };
        match classification {
            RealRootFiberCertificate::Nonempty { root, .. } => values.push(root.clone()),
            RealRootFiberCertificate::Empty { root, .. } => rejected_roots.push(root.clone()),
        }
    }

    Some(CertifiedExactTargetImage {
        support,
        squarefree_support: squarefree_support.clone(),
        values,
        rejected_roots,
        certificate: ExactTargetImageCertificate {
            cover,
            squarefree_support,
            root_classifications,
        },
    })
}

fn classified_root(classification: &RealRootFiberCertificate) -> &AlgebraicRealRoot {
    match classification {
        RealRootFiberCertificate::Nonempty { root, .. }
        | RealRootFiberCertificate::Empty { root, .. } => root,
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use num_rational::BigRational;

    use super::*;
    use crate::{
        ExactIdentity, ExactIdentityKind, Monomial, PolynomialQ, Rational, TargetCertificate,
        Variable,
    };

    fn variable(symbol: &str) -> Variable {
        Variable {
            symbol: symbol.to_string(),
        }
    }

    fn rational(value: i64) -> Rational {
        BigRational::from_integer(BigInt::from(value))
    }

    fn uni(variable: &Variable, coefficients: &[i64]) -> UniPolynomialQ {
        let mut polynomial = UniPolynomialQ {
            variable: variable.clone(),
            coefficients: coefficients.iter().map(|value| rational(*value)).collect(),
        };
        polynomial.normalize();
        polynomial
    }

    fn root(
        polynomial: &UniPolynomialQ,
        lower: i64,
        upper: i64,
        index: usize,
    ) -> AlgebraicRealRoot {
        AlgebraicRealRoot {
            polynomial: polynomial.clone(),
            isolating_interval: crate::RationalInterval {
                lower: rational(lower),
                upper: rational(upper),
            },
            index,
        }
    }

    fn certificate(support: UniPolynomialQ) -> TargetCertificate {
        let variables = vec![support.variable.clone()];
        TargetCertificate::IdealMembership {
            support,
            multipliers: vec![PolynomialQ::from_term(
                variables,
                rational(1),
                Monomial { exponents: vec![0] },
            )],
            identity: ExactIdentity {
                kind: ExactIdentityKind::IdealMembership,
            },
        }
    }

    #[test]
    fn exact_image_requires_every_root_classified() {
        let t = variable("T");
        let support = uni(&t, &[-2, 0, 1]);
        let roots = vec![root(&support, -2, -1, 0), root(&support, 1, 2, 1)];
        let classifications = vec![RealRootFiberCertificate::Nonempty {
            root: roots[0].clone(),
            certificate: RealFiberNonemptyCertificate {
                description: "exact nonempty certificate".to_string(),
            },
        }];

        let image = build_certified_exact_target_image(
            certificate(support.clone()),
            support.clone(),
            support,
            &roots,
            classifications,
        );

        assert!(image.is_none());
    }
}
