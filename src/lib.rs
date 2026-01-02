#![no_main]
#![no_std]

extern crate cortex_m;
use daisy::{
    self,
    hal::{prelude::*, adc, delay, gpio},
    pac,
    Board,
};

pub struct Hothouse {
    board: Board,
    knob_adc: adc::Adc<pac::ADC1, adc::Enabled>,
    knobs: [gpio::ErasedPin<gpio::Analog>; 6],
}

impl Hothouse {
    pub fn take() -> Self {
        let cp = cortex_m::Peripherals::take().unwrap();
        let dp = pac::Peripherals::take().unwrap();
        let board = Board::take().unwrap();

        let ccdr = daisy::board_freeze_clocks!(board, dp);
        let pins = daisy::board_split_gpios!(board, ccdr, dp);

        let knobs = [
            pins.GPIO.PIN_16.into_analog().erase(),
            pins.GPIO.PIN_17.into_analog().erase(),
            pins.GPIO.PIN_18.into_analog().erase(),
            pins.GPIO.PIN_19.into_analog().erase(),
            pins.GPIO.PIN_20.into_analog().erase(),
            pins.GPIO.PIN_21.into_analog().erase(),
        ];

        // Configure ADC.
        let mut delay = delay::Delay::new(cp.SYST, ccdr.clocks);
        let mut knob_adc = adc::Adc::adc1(
            dp.ADC1,
            4.MHz(),
            &mut delay,
            ccdr.peripheral.ADC12,
            &ccdr.clocks,
        )
        .enable();
        knob_adc.set_resolution(adc::Resolution::SixteenBit);

        Self { board, knob_adc, knobs }
    }
}
