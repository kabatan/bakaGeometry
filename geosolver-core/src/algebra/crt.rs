use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModInteger {
    pub value: BigInt,
    pub modulus: BigInt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModVector {
    pub entries: Vec<BigInt>,
    pub modulus: BigInt,
}

pub fn crt_combine(a_mod_m: ModInteger, b_mod_n: ModInteger) -> ModInteger {
    try_crt_combine(&a_mod_m, &b_mod_n)
        .expect("CRT inputs must be compatible and have positive moduli")
}

pub fn crt_vector_combine(v1: ModVector, mod1: BigInt, v2: ModVector, mod2: BigInt) -> ModVector {
    try_crt_vector_combine(&v1, &mod1, &v2, &mod2)
        .expect("CRT vector inputs must be compatible and have matching length")
}

pub fn try_crt_vector_combine(
    v1: &ModVector,
    mod1: &BigInt,
    v2: &ModVector,
    mod2: &BigInt,
) -> Option<ModVector> {
    if v1.entries.len() != v2.entries.len() {
        return None;
    }
    let entries = v1
        .entries
        .iter()
        .zip(v2.entries.iter())
        .map(|(a, b)| {
            try_crt_combine(
                &ModInteger {
                    value: a.clone(),
                    modulus: mod1.clone(),
                },
                &ModInteger {
                    value: b.clone(),
                    modulus: mod2.clone(),
                },
            )
            .map(|combined| combined.value)
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ModVector {
        entries,
        modulus: mod1.lcm(mod2),
    })
}

pub fn try_crt_combine(a_mod_m: &ModInteger, b_mod_n: &ModInteger) -> Option<ModInteger> {
    if a_mod_m.modulus <= BigInt::zero() || b_mod_n.modulus <= BigInt::zero() {
        return None;
    }
    let m = &a_mod_m.modulus;
    let n = &b_mod_n.modulus;
    let a = canonical_mod(&a_mod_m.value, m);
    let b = canonical_mod(&b_mod_n.value, n);
    let g = m.gcd(n);
    let diff = &b - &a;
    if diff.mod_floor(&g) != BigInt::zero() {
        return None;
    }
    let m_g = m / &g;
    let n_g = n / &g;
    let inv = mod_inverse_bigint(&m_g, &n_g)?;
    let t = canonical_mod(&(diff / &g * inv), &n_g);
    let lcm = m * &n_g;
    Some(ModInteger {
        value: canonical_mod(&(a + m * t), &lcm),
        modulus: lcm,
    })
}

pub fn canonical_mod(value: &BigInt, modulus: &BigInt) -> BigInt {
    let mut r = value.mod_floor(modulus);
    if r.is_negative() {
        r += modulus;
    }
    r
}

fn mod_inverse_bigint(a: &BigInt, modulus: &BigInt) -> Option<BigInt> {
    let (g, x, _) = extended_gcd(a.clone(), modulus.clone());
    if g != BigInt::one() {
        return None;
    }
    Some(canonical_mod(&x, modulus))
}

fn extended_gcd(a: BigInt, b: BigInt) -> (BigInt, BigInt, BigInt) {
    if b.is_zero() {
        return (a.abs(), a.signum(), BigInt::zero());
    }
    let (g, x1, y1) = extended_gcd(b.clone(), a.mod_floor(&b));
    let x = y1.clone();
    let y = x1 - (a / b) * y1;
    (g, x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crt_round_trip_combines_coprime_moduli() {
        let combined = crt_combine(
            ModInteger {
                value: BigInt::from(2),
                modulus: BigInt::from(3),
            },
            ModInteger {
                value: BigInt::from(3),
                modulus: BigInt::from(5),
            },
        );
        assert_eq!(combined.value, BigInt::from(8));
        assert_eq!(combined.modulus, BigInt::from(15));
    }

    #[test]
    fn incompatible_crt_returns_none_in_checked_api() {
        assert!(try_crt_combine(
            &ModInteger {
                value: BigInt::from(1),
                modulus: BigInt::from(4),
            },
            &ModInteger {
                value: BigInt::from(2),
                modulus: BigInt::from(6),
            },
        )
        .is_none());
    }

    #[test]
    fn checked_vector_crt_rejects_mismatch_and_incompatibility() {
        let v1 = ModVector {
            entries: vec![BigInt::from(1), BigInt::from(2)],
            modulus: BigInt::from(4),
        };
        let short = ModVector {
            entries: vec![BigInt::from(1)],
            modulus: BigInt::from(5),
        };
        assert!(try_crt_vector_combine(&v1, &BigInt::from(4), &short, &BigInt::from(5),).is_none());

        let incompatible = ModVector {
            entries: vec![BigInt::from(2), BigInt::from(2)],
            modulus: BigInt::from(6),
        };
        assert!(
            try_crt_vector_combine(&v1, &BigInt::from(4), &incompatible, &BigInt::from(6),)
                .is_none()
        );
    }
}
