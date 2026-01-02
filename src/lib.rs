#![no_main]
#![no_std]

extern crate cortex_m;
use daisy::{self, pac, Board};
use stm32h7xx_hal::{prelude::*, adc};

pub struct Hothouse {
    cp: cortex_m::Peripherals,
    dp: pac::Peripherals,
    board: Board,
}
