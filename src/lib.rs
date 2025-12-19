#![recursion_limit = "256"]

use {
    debug_err::{DebugErr, src},
    std::{fmt, str::FromStr},
};

type Result<T> = std::result::Result<T, DebugErr>;

mod stack;

#[macro_export]
macro_rules! dec {
    ($lit:literal) => {
        $crate::Decimal::from_str(stringify!($lit)).unwrap()
    };
}

#[macro_export]
macro_rules! rpn {
    ($($rest:tt)*) => { rpn_inner!( Stack::new() , $($rest)*) };
}

#[doc(hidden)]
#[macro_export]
macro_rules! rpn_inner {

    // No more commas
    ($stack:expr) => { $stack };

    // If there is only one command remaining
     
    ($stack:expr , +) => { $stack.op2(Op2::Add) };

    ($stack:expr , -) => { $stack.op2(Op2::Sub) };

    ($stack:expr , *) => { $stack.op2(Op2::Mul) };

    ($stack:expr , /) => { $stack.op2(Op2::Div) };

    ($stack:expr , $val:expr) => { $stack.enter($val) };

    // We want to work through the first token in $rest.

    ($stack:expr , + , $($rest:tt)*) => { rpn_inner!($stack.op2(Op2::Add) , $($rest)*) };

    ($stack:expr , - , $($rest:tt)*) => { rpn_inner!($stack.op2(Op2::Sub) , $($rest)*) };

    ($stack:expr , *, $($rest:tt)*) => { rpn_inner!($stack.op2(Op2::Mul) , $($rest)*) };
    
    ($stack:expr , / , $($rest:tt)*) => { rpn_inner!($stack.op2(Op2::Div) , $($rest)*) };

    ($stack:expr , $val:expr , $($rest:tt)*) => { rpn_inner!($stack.enter($val) , $($rest)*) };
}

pub trait DecimalOps where
    Self: Clone + Copy + Sized + PartialEq,
{

    /**
    ### Errors
    */
    fn new(num: i64, scale: u32) -> Result<Self>;

    /**
    ### Errors
    */
    fn add(self, other: Self) -> Result<Self>;

    /**
    ### Errors
    */
    fn sub(self, other: Self) -> Result<Self>;

    /**
    ### Errors
    */
    fn mul(self, other: Self) -> Result<Self>; 

    /**
    ### Errors
    */
    fn div(self, other: Self) -> Result<Self>;

    fn is_zero(&self) -> bool;

    fn zero() -> Self;

    fn change_sign(&self) -> Self;

    fn sqrt(&self) -> Result<Self>;

    /**
    ### Errors
    */
    fn cos(&self) -> Result<Self>;

    /**
    ### Errors
    */
    fn sin(&self) -> Result<Self>;
}

impl DecimalOps for rust_decimal::Decimal {

    fn add(self, other: Self) -> Result<Self> {
        self.checked_add(other).ok_or_else(|| src!("Overflow/underflow on addition"))
    }

    fn sub(self, other: Self) -> Result<Self> {
        self.checked_sub(other).ok_or_else(|| src!("Overflow/underflow on addition"))
    }

    fn mul(self, other: Self) -> Result<Self> {
        self.checked_mul(other).ok_or_else(|| src!("Overflow/underflow on addition"))
    }

    fn div(self, other: Self) -> Result<Self> {
        if other.is_zero() { Err(src!("Division by zero"))? }
        self.checked_div(other).ok_or_else(|| src!("Overflow/underflow on addition"))
    }

    fn new(num: i64, scale: u32) -> Result<Self> {
        Self::try_new(num, scale).map_err(|e| src!("{e}"))
    }

    fn is_zero(&self) -> bool {
        Self::is_zero(self)
    }

    fn zero() -> Self {
        Self::ZERO
    }

    fn change_sign(&self) -> Self {
        -self
    }

    fn sqrt(&self) -> Result<Self> {
        rust_decimal::MathematicalOps::sqrt(self).ok_or_else(|| src!("sqrt failed"))
    }

    fn cos(&self) -> Result<Self> {
        rust_decimal::MathematicalOps::checked_cos(self).ok_or_else(|| src!("Failed to find cos"))
    }

    fn sin(&self) -> Result<Self> {
        rust_decimal::MathematicalOps::checked_sin(self).ok_or_else(|| src!("Failed to find sin"))
    }
}

/**
This is the Decimal type we present to the rest of the crate, and externally. If we
want to change the inner type we'll need to reimplement these functions.
*/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decimal(pub(crate) rust_decimal::Decimal);

impl FromStr for Decimal {
    type Err = DebugErr;

    fn from_str(s: &str) -> Result<Self> {
        let inner = rust_decimal::Decimal::from_str(s).map_err(|e| src!("{e}"))?;
        Ok(Self(inner))
    }    
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl DecimalOps for Decimal {

    fn new(num: i64, scale: u32) -> Result<Self> {
        Ok(Self(DecimalOps::new(num, scale)?))
    }



    fn add(self, other: Self) -> Result<Self> {
        Ok(Self(self.0.add(other.0)?))
    }

    fn sub(self, other: Self) -> Result<Self> {
        Ok(Self(self.0.sub(other.0)?))
    }

    fn mul(self, other: Self) -> Result<Self> {
        Ok(Self(self.0.mul(other.0)?))
    }

    fn div(self, other: Self) -> Result<Self> {
        Ok(Self(self.0.div(other.0)?))
    }

    fn is_zero(&self) -> bool { self.0.is_zero() }

    fn zero() -> Self { Self(rust_decimal::Decimal::zero()) }

    fn change_sign(&self) -> Self { Self(self.0.change_sign()) }

    fn sqrt(&self) -> Result<Self> { Ok(Self(self.0.sqrt()?)) }

    fn cos(&self) -> Result<Self> { Ok(Self(self.0.cos()?)) }

    fn sin(&self) -> Result<Self> { Ok(Self(self.0.sin()?)) }
}

