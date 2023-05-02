#![cfg_attr(not(test), no_std)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]



#[cfg(not(feature = "device-selected"))]
core::compile_error!(
    "This crate requires one of the following device features enabled:
    matekh743
"
);

#[cfg(feature = "matekh743")]
pub mod MatekH743;
#[cfg(feature = "matekh743")]
pub use MatekH743 as periph;
#[cfg(feature = "matekh743")]
pub use stm32h7xx_hal as hal;

#[cfg(feature = "matekf411")]
use stm32f4xx_hal as hal;
#[cfg(feature = "matekf411")]
pub mod MatekF411;
#[cfg(feature = "matekf411")]
pub use MatekF411 as periph;

// pub mod led;
// pub mod serial;
