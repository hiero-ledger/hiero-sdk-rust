// SPDX-License-Identifier: Apache-2.0

use fraction::GenericFraction;

type Fraction = GenericFraction<u64>;

impl From<super::proto::proto::Fraction> for Fraction {
    fn from(pb: super::proto::proto::Fraction) -> Self {
        Fraction::new(pb.numerator as u64, pb.denominator as u64)
    }
}

impl From<Fraction> for super::proto::proto::Fraction {
    fn from(frac: Fraction) -> Self {
        Self {
            numerator: frac.numer().copied().unwrap_or_default() as i64,
            denominator: frac.denom().copied().unwrap_or_default() as i64,
        }
    }
}

