use bigdecimal::ToPrimitive;
use ethers::prelude::*;

pub trait Decimals {
    fn to_decimals(&self, decimals: u8) -> U256;
    fn from_decimals(&self, decimals: u8) -> f64;
}

impl Decimals for f64 {
    fn to_decimals(&self, decimals: u8) -> U256 {
        return U256::from(
            (self
                * 10_f64.powi(
                    decimals
                        .to_i32()
                        .expect("f64.to_decimals(): Failed to convert decimals to i32"),
                ))
            .to_u128()
            .expect("f64.to_decimals(): Failed to convert f64 to u128"),
        );
    }

    fn from_decimals(&self, decimals: u8) -> f64 {
        return self
            / 10_f64.powi(
                decimals
                    .to_i32()
                    .expect("f64.from_decimals(): Failed to convert decimals to i32"),
            );
    }
}

impl Decimals for u64 {
    fn to_decimals(&self, decimals: u8) -> U256 {
        return U256::from(*self) * U256::from(10).pow(decimals.into());
    }

    fn from_decimals(&self, decimals: u8) -> f64 {
        return (*self as f64)
            / 10_f64.powi(
                decimals
                    .to_i32()
                    .expect("u64.from_decimals(): Failed to convert decimals to i32"),
            );
    }
}

impl Decimals for usize {
    fn to_decimals(&self, decimals: u8) -> U256 {
        return U256::from(*self) * U256::from(10).pow(decimals.into());
    }

    fn from_decimals(&self, decimals: u8) -> f64 {
        return (*self as f64)
            / 10_f64.powi(
                decimals
                    .to_i32()
                    .expect("usize.from_decimals(): Failed to convert decimals to i32"),
            );
    }
}

impl Decimals for U256 {
    fn to_decimals(&self, decimals: u8) -> U256 {
        return self * U256::from(10).pow(decimals.into());
    }

    fn from_decimals(&self, decimals: u8) -> f64 {
        return (self.as_u128() as f64)
            / 10_f64.powi(
                decimals
                    .to_i32()
                    .expect("U256.from_decimals(): Failed to convert decimals to i32"),
            );
    }
}
