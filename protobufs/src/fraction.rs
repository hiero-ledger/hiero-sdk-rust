// SPDX-License-Identifier: Apache-2.0

use fraction::Fraction;

impl From<super::services::Fraction> for Fraction {
    fn from(pb: super::services::Fraction) -> Self {
        Fraction::new(pb.numerator as u64, pb.denominator as u64)
    }
}

impl From<Fraction> for super::services::Fraction {
    fn from(frac: Fraction) -> Self {
        Self {
            numerator: frac.numer().copied().unwrap_or_default() as i64,
            denominator: frac.denom().copied().unwrap_or_default() as i64,
        }
    }
}
