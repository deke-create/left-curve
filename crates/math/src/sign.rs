use {
    crate::{Dec, Int, NumberConst},
    bnum::types::{I256, I512, U256, U512},
};

/// Describes a number that can take on negative values.
/// Zero is considered non-negative, for which this should return `false`.
pub trait Sign {
    fn abs(self) -> Self;

    fn is_negative(&self) -> bool;
}

// ------------------------------------ int ------------------------------------

impl<U> Sign for Int<U>
where
    U: Sign,
{
    fn abs(self) -> Self {
        Self(self.0.abs())
    }

    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

// ------------------------------------ dec ------------------------------------

impl<U> Sign for Dec<U>
where
    U: Sign,
{
    fn abs(self) -> Self {
        Self(self.0.abs())
    }

    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

// ----------------------------------- unsigned ------------------------------------

macro_rules! impl_sign_unsigned {
    ($t:ty) => {
        impl Sign for $t {
            fn abs(self) -> Self {
                self
            }

            fn is_negative(&self) -> bool {
                false
            }
        }
    };
    ($($t:ty),+ $(,)?) => {
        $(
            impl_sign_unsigned!($t);
        )+
    };
}

impl_sign_unsigned!(u8, u16, u32, u64, u128, U256, U512);

// ---------------------------------- signed -----------------------------------

macro_rules! impl_sign_signed {
    ($t:ty) => {
        impl Sign for $t {
            fn abs(self) -> Self {
                self.abs()
            }

            fn is_negative(&self) -> bool {
                *self < Self::ZERO
            }
        }
    };
    ($($t:ty),+ $(,)?) => {
        $(
            impl_sign_signed!($t);
        )+
    };
}

impl_sign_signed!(i8, i16, i32, i64, i128, I256, I512);