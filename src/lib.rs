#![no_main]
#![no_std]

extern crate cortex_m;
use daisy::{
    self,
    hal::{prelude::*, adc, delay, gpio},
    pac,
    Board,
};

pub enum HothouseError {
    BadIndex,
}

struct Knobs {
    knob1: gpio::gpioa::PA3<gpio::Analog>,
    knob2: gpio::gpiob::PB1<gpio::Analog>,
    knob3: gpio::gpioa::PA7<gpio::Analog>,
    knob4: gpio::gpioa::PA6<gpio::Analog>,
    knob5: gpio::gpioc::PC1<gpio::Analog>,
    knob6: gpio::gpioc::PC4<gpio::Analog>,
}

pub struct Hothouse {
    board: Board,
    knob_adc: adc::Adc<pac::ADC1, adc::Enabled>,
    knobs: Knobs,
}

impl Hothouse {
    pub fn take() -> Self {
        let cp = cortex_m::Peripherals::take().unwrap();
        let dp = pac::Peripherals::take().unwrap();
        let board = Board::take().unwrap();

        let ccdr = daisy::board_freeze_clocks!(board, dp);
        let pins = daisy::board_split_gpios!(board, ccdr, dp);

        let knobs = Knobs {
            knob1: pins.GPIO.PIN_16.into_analog(),
            knob2: pins.GPIO.PIN_17.into_analog(),
            knob3: pins.GPIO.PIN_18.into_analog(),
            knob4: pins.GPIO.PIN_19.into_analog(),
            knob5: pins.GPIO.PIN_20.into_analog(),
            knob6: pins.GPIO.PIN_21.into_analog(),
        };

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

    pub fn read_knob(&mut self, knob_id: usize) -> Result<f32, HothouseError> {
        let position: u32 = match knob_id {
            1 => self.knob_adc.read(&mut self.knobs.knob1),
            2 => self.knob_adc.read(&mut self.knobs.knob2),
            3 => self.knob_adc.read(&mut self.knobs.knob3),
            4 => self.knob_adc.read(&mut self.knobs.knob4),
            5 => self.knob_adc.read(&mut self.knobs.knob5),
            6 => self.knob_adc.read(&mut self.knobs.knob6),
            _ => return Err(HothouseError::BadIndex),
        }.unwrap();
        Ok(position as f32 / 65_635.0)
    }
}
