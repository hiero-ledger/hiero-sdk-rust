use std::fmt::{
    Debug,
    Display,
    Formatter,
};
use std::ops;
use std::str::FromStr;

use rust_decimal::prelude::*;

use crate::Error;

/// Type alias for `i64` representing `tinybar`
pub type Tinybar = i64;

/// Common units of hbar; for the most part they follow SI prefix conventions.
///
/// See the [Hiero Documentation](https://docs.hedera.com/guides/docs/sdks/hbars#hbar-units).
#[repr(i64)]
#[derive(Debug, Copy, Hash, PartialEq, Eq, Clone)]
pub enum HbarUnit {
    /// The atomic (smallest) unit of [`Hbar`], used natively by the Hiero network.
    ///
    /// It is equivalent to <sup>1</sup>&frasl;<sub>100,000,000</sub> [`Hbar`](Self::Hbar).
    Tinybar = 1,

    /// Equivalent to 100 [`Tinybar`](Self::Tinybar) or <sup>1</sup>&frasl;<sub>1,000,000</sub> [`Hbar`](Self::Hbar).
    Microbar = 100,

    /// Equivalent to 100,000 [`Tinybar`](Self::Tinybar) or <sup>1</sup>&frasl;<sub>1,000</sub> [`Hbar`](Self::Hbar).
    Millibar = 100_000,

    /// The base unit of [`Hbar`], equivalent to 100 million [`Tinybar`](Self::Tinybar).
    Hbar = 100_000_000,

    /// Equivalent to 1 thousand [`Hbar`](Self::Hbar) or 100 billion [`Tinybar`](Self::Tinybar).
    Kilobar = 1_000 * 100_000_000,

    /// Equivalent to 1 million [`Hbar`](Self::Hbar) or 100 trillion [`Tinybar`](Self::Tinybar).
    Megabar = 1_000_000 * 100_000_000,

    /// Equivalent to 1 billion [`Hbar`](Self::Hbar) or 100 quadrillion [`Tinybar`](Self::Tinybar).
    ///
    /// The maximum hbar amount supported by Hiero in any context is ~92 gigabar
    /// (2<sup>63</sup> tinybar); use this unit sparingly.
    Gigabar = 1_000_000_000 * 100_000_000,
}

impl HbarUnit {
    /// Returns the the value of `self` in `Tinybar`.
    ///
    /// # Examples
    /// ```
    /// use hedera::HbarUnit;
    /// assert_eq!(HbarUnit::Microbar.tinybars(), 100);
    /// ```
    #[must_use]
    pub const fn tinybars(self) -> Tinybar {
        self as Tinybar
    }

    /// Returns a `str` containing the symbol for `self`.
    ///
    /// # Examples
    /// ```
    /// use hedera::HbarUnit;
    /// assert_eq!(HbarUnit::Millibar.symbol(), "mℏ");
    /// ```
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            HbarUnit::Tinybar => "tℏ",
            HbarUnit::Microbar => "μℏ",
            HbarUnit::Millibar => "mℏ",
            HbarUnit::Hbar => "ℏ",
            HbarUnit::Kilobar => "kℏ",
            HbarUnit::Megabar => "Mℏ",
            HbarUnit::Gigabar => "Gℏ",
        }
    }
}

impl Display for HbarUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.symbol())
    }
}

impl FromStr for HbarUnit {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tℏ" => Ok(HbarUnit::Tinybar),
            "μℏ" => Ok(HbarUnit::Microbar),
            "mℏ" => Ok(HbarUnit::Millibar),
            "ℏ" => Ok(HbarUnit::Hbar),
            "kℏ" => Ok(HbarUnit::Kilobar),
            "Mℏ" => Ok(HbarUnit::Megabar),
            "Gℏ" => Ok(HbarUnit::Gigabar),
            _ => Err(Error::basic_parse(format!(
                "Given string `{s}` was not recognized as an Hbar unit symbol"
            ))),
        }
    }
}

/// A quantity of `hbar`.
#[derive(Default, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Hbar(i64);

impl Hbar {
    /// A constant value of `0 ℏ`.
    pub const ZERO: Hbar = Hbar::from_tinybars(0);

    /// The minimum allowable amount of hbar `-50 Gℏ`
    pub const MIN: Hbar = Self::from_integer_unit(-50, HbarUnit::Gigabar);

    /// The maximum allowable amount of hbar `50 Gℏ`.
    pub const MAX: Self = Self::from_integer_unit(50, HbarUnit::Gigabar);

    /// Create a `Hbar` containing `amount ℏ`.
    ///
    /// # Examples
    /// ```
    /// # use hedera::Hbar;
    /// let hbar = Hbar::new(52);
    /// assert_eq!(hbar.to_string(), "52 ℏ");
    #[must_use]
    pub const fn new(amount: i64) -> Self {
        Self::from_integer_unit(amount, HbarUnit::Hbar)
    }

    /// Convert from `tinybars` to `Hbar`.
    ///
    /// # Examples
    /// ```
    /// # use hedera::Hbar;
    /// let hbar = Hbar::from_tinybars(250);
    /// assert_eq!(hbar.to_string(), "250 tℏ");
    /// ```
    #[must_use]
    pub const fn from_tinybars(tinybars: Tinybar) -> Self {
        // todo: `debug_assert!` or `assert!` in range?
        Hbar(tinybars)
    }

    /// Helper for things like `20 Gℏ -> Hbar`.
    const fn from_integer_unit(amount: i64, unit: HbarUnit) -> Self {
        Self::from_tinybars(amount * unit.tinybars())
    }

    // fixme(sr): poor wording on `Truncates...`
    /// Convert from `amount` in `unit` to `Hbar`.
    ///
    /// Truncates `amount` to the nearest tinybar if the resulting `Hbar` is not an integer amount of tinybar.
    ///
    /// # Panics
    ///
    /// * if `amount * unit.tinybars()` would overflow a i64.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedera::Hbar;
    /// use hedera::HbarUnit;
    /// let value = Hbar::from_unit(20, HbarUnit::Millibar);
    /// assert_eq!(value.to_string(), "0.02 ℏ");
    /// ```
    #[must_use]
    #[track_caller]
    pub fn from_unit<T>(amount: T, unit: HbarUnit) -> Self
    where
        T: Into<Decimal>,
    {
        let unit_tinybars: Decimal = unit.tinybars().into();
        let amount_tinybars = amount.into().checked_mul(unit_tinybars).unwrap();

        Hbar::from_tinybars(amount_tinybars.to_i64().unwrap())
    }

    /// Returns the value of `self` in `Tinybar`s.
    #[must_use]
    pub const fn to_tinybars(self) -> Tinybar {
        self.0
    }

    /// Returns `self` as `Decimal` `unit`s.
    #[must_use]
    pub fn to(self, unit: HbarUnit) -> Decimal {
        Decimal::from(self.to_tinybars()) / Decimal::from(unit.tinybars())
    }

    /// Returns `self` as `Decimal` hbars.
    ///
    /// # Examples
    /// ```
    /// use rust_decimal::Decimal;
    /// use hedera::Hbar;
    /// # use std::str::FromStr;
    /// let value: Hbar = "20 mℏ".parse().unwrap();
    ///
    /// let value_decimal: Decimal = "0.02".parse().unwrap();
    ///
    /// assert_eq!(value.get_value(), value_decimal);
    /// ```
    #[must_use]
    pub fn get_value(self) -> Decimal {
        self.to(HbarUnit::Hbar)
    }

    /// Returns [`-self`](std::ops::Neg::neg).
    #[must_use]
    pub fn negated(self) -> Self {
        -self
    }
}

impl From<Hbar> for Decimal {
    fn from(hbar: Hbar) -> Self {
        hbar.get_value()
    }
}

impl From<Decimal> for Hbar {
    fn from(hbars: Decimal) -> Self {
        Hbar::from_unit(hbars, HbarUnit::Hbar)
    }
}

impl Display for Hbar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.to_tinybars() > -10_000 && self.to_tinybars() < 10_000 {
            write!(f, "{} {}", self.to_tinybars(), HbarUnit::Tinybar.symbol())
        } else {
            write!(f, "{} {}", self.to(HbarUnit::Hbar), HbarUnit::Hbar.symbol())
        }
    }
}

impl Debug for Hbar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl FromStr for Hbar {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (amount, unit) = s.split_once(' ').unwrap_or((s, "ℏ"));
        let amount: Decimal = amount.parse().map_err(Error::basic_parse)?;
        let unit = HbarUnit::from_str(unit)?;
        Ok(Hbar::from_unit(amount, unit))
    }
}

impl ops::Neg for Hbar {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl ops::Add for Hbar {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::AddAssign for Hbar {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl ops::Sub for Hbar {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl ops::SubAssign for Hbar {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl<T> ops::Mul<T> for Hbar
where
    i64: ops::Mul<T, Output = i64>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl<T> ops::MulAssign<T> for Hbar
where
    i64: ops::MulAssign<T>,
{
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs;
    }
}

impl<T> ops::Div<T> for Hbar
where
    i64: ops::Div<T, Output = i64>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl<T> ops::DivAssign<T> for Hbar
where
    i64: ops::DivAssign<T>,
{
    fn div_assign(&mut self, rhs: T) {
        self.0 /= rhs;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rust_decimal::Decimal;

    use crate::{
        Hbar,
        HbarUnit,
    };

    #[test]
    fn it_can_parse() {
        assert_eq!(Hbar::from_str("10 tℏ").unwrap(), Hbar::from_tinybars(10));
        assert_eq!(Hbar::from_str("11 μℏ").unwrap(), Hbar::from_unit(11, HbarUnit::Microbar));
        assert_eq!(Hbar::from_str("12 mℏ").unwrap(), Hbar::from_unit(12, HbarUnit::Millibar));
        assert_eq!(Hbar::from_str("13 ℏ").unwrap(), Hbar::from_unit(13, HbarUnit::Hbar));
        assert_eq!(Hbar::from_str("14 kℏ").unwrap(), Hbar::from_unit(14, HbarUnit::Kilobar));
        assert_eq!(Hbar::from_str("15 Mℏ").unwrap(), Hbar::from_unit(15, HbarUnit::Megabar));
        assert_eq!(Hbar::from_str("16 Gℏ").unwrap(), Hbar::from_unit(16, HbarUnit::Gigabar));
        assert_eq!(Hbar::from_str("17").unwrap(), Hbar::from(Decimal::from(17)));
        assert_eq!(Hbar::from_str("-17 ℏ").unwrap(), Hbar::new(-17));
        assert_eq!(Hbar::from_str("+19 ℏ").unwrap(), Hbar::new(19));
    }

    #[test]
    fn it_can_to_string() {
        assert_eq!(Hbar::from_unit(9_999, HbarUnit::Tinybar).to_string(), "9999 tℏ");
        assert_eq!(Hbar::from_unit(10_000, HbarUnit::Tinybar).to_string(), "0.0001 ℏ");
        assert_eq!(Hbar::from_unit(-9_999, HbarUnit::Tinybar).to_string(), "-9999 tℏ");
        assert_eq!(Hbar::from_unit(-10_000, HbarUnit::Tinybar).to_string(), "-0.0001 ℏ");
    }

    #[test]
    fn it_can_arithmatic() {
        let ten = Hbar::from_tinybars(10);
        let three = Hbar::from_tinybars(3);
        let one = Hbar::from_tinybars(1);

        assert_eq!((ten * 2) - (ten / 2) + three, Hbar::from_tinybars((10 * 2) - (10 / 2) + 3));

        let mut m = three;
        m *= 2;
        assert_eq!(m.to_tinybars(), 6);
        m /= 2;
        assert_eq!(m.to_tinybars(), 3);
        m += one;
        assert_eq!(m.to_tinybars(), 4);
        m -= one;
        assert_eq!(m.to_tinybars(), 3);
        assert_eq!((-m).to_tinybars(), -3);
    }
}
