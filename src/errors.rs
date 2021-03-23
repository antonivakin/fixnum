use core::fmt::{Display, Formatter, Result};

#[cfg(feature = "std")]
use derive_more::Error;

macro_rules! impl_error {
    ($err:ident) => {
        impl Display for $err {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                f.write_str(self.as_str())
            }
        }

        #[cfg(test)]
        impl From<$err> for anyhow::Error {
            fn from(err: $err) -> Self {
                Self::msg(err.as_str())
            }
        }
    };
}

#[cfg_attr(feature = "std", derive(Error))]
#[derive(Clone, Debug, PartialEq)]
pub enum ArithmeticError {
    Overflow,
    DivisionByZero,
    /// When someone tries to use operand out of the set of departure of the function.
    /// E.g.: when you try to compute the square root of a negative number.
    DomainViolation,
}

impl ArithmeticError {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Overflow => "overflow",
            Self::DivisionByZero => "division by zero",
            Self::DomainViolation => "domain violation",
        }
    }
}

impl_error!(ArithmeticError);

#[cfg_attr(feature = "std", derive(Error))]
#[derive(Clone, Debug, PartialEq)]
pub enum FromDecimalError {
    UnsupportedExponent,
    TooBigMantissa,
}

impl FromDecimalError {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedExponent => "unsupported exponent",
            Self::TooBigMantissa => "mantissa is too big",
        }
    }
}

impl_error!(FromDecimalError);

#[cfg_attr(feature = "std", derive(Error))]
#[derive(Clone, Debug, PartialEq)]
pub enum ConvertError {
    Overflow,
    Other(#[cfg_attr(feature = "std", error(not(source)))] &'static str),
}

impl ConvertError {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Overflow => "unsupported exponent",
            Self::Other(_) => "mantissa is too big",
        }
    }
}

impl_error!(ConvertError);
