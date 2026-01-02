#![no_main]
#![no_std]

extern crate cortex_m;
pub use daisy as board;
pub use daisy::hal;
pub use daisy::pac;
use daisy::{
    hal::{prelude::*, adc, delay, gpio},
    Board,
};

pub enum HothouseError {
    BadIndex,
    SwitchFailure,
}

pub struct Knobs {
    pub knob1: gpio::gpioa::PA3<gpio::Analog>,
    pub knob2: gpio::gpiob::PB1<gpio::Analog>,
    pub knob3: gpio::gpioa::PA7<gpio::Analog>,
    pub knob4: gpio::gpioa::PA6<gpio::Analog>,
    pub knob5: gpio::gpioc::PC1<gpio::Analog>,
    pub knob6: gpio::gpioc::PC4<gpio::Analog>,
}

pub struct Spdt<Down, Up> {
    pub down: Down,
    pub up: Up,
}

pub struct Toggles {
    pub sw1: Spdt<gpio::gpiob::PB5<gpio::Input>, gpio::gpiob::PB4<gpio::Input>>,
    pub sw2: Spdt<gpio::gpiog::PG11<gpio::Input>, gpio::gpiog::PG10<gpio::Input>>,
    pub sw3: Spdt<gpio::gpioc::PC12<gpio::Input>, gpio::gpiod::PD2<gpio::Input>>,
}

pub enum ToggleState {
    Down,
    Centered,
    Up,
}

pub struct Leds {
    pub led1: gpio::gpioa::PA5<gpio::Output>,
    pub led2: gpio::gpioa::PA4<gpio::Output>,
}

pub struct Footswitches {
    pub fsw1: gpio::gpioa::PA0<gpio::Input>,
    pub fsw2: gpio::gpiod::PD11<gpio::Input>,
}

pub struct Hothouse {
    pub board: Board,
    knob_adc: adc::Adc<pac::ADC1, adc::Enabled>,
    pub knobs: Knobs,
    pub leds: Leds,
    pub toggles: Toggles,
    pub footswitches: Footswitches,
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

        let toggles = Toggles {
            sw1: Spdt {
                down: pins.GPIO.PIN_10.into_pull_up_input(),
                up: pins.GPIO.PIN_9.into_pull_up_input(),
            },
            sw2: Spdt {
                down: pins.GPIO.PIN_8.into_pull_up_input(),
                up: pins.GPIO.PIN_7.into_pull_up_input(),
            },
            sw3: Spdt {
                down: pins.GPIO.PIN_6.into_pull_up_input(),
                up: pins.GPIO.PIN_5.into_pull_up_input(),
            },
        };

        let leds = Leds {
            led1: pins.GPIO.PIN_22.into_push_pull_output(),
            led2: pins.GPIO.PIN_23.into_push_pull_output(),
        };

        let footswitches = Footswitches {
            fsw1: pins.GPIO.PIN_25.into_pull_up_input(),
            fsw2: pins.GPIO.PIN_26.into_pull_up_input(),
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

        Self { board, knob_adc, knobs, toggles, leds, footswitches }
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

    pub fn read_toggle(&mut self, toggle_id: usize) -> Result<ToggleState, HothouseError> {
        let sets = match toggle_id {
            1 => (self.toggles.sw1.down.is_high(), self.toggles.sw1.up.is_high()),
            2 => (self.toggles.sw2.down.is_high(), self.toggles.sw2.up.is_high()),
            3 => (self.toggles.sw3.down.is_high(), self.toggles.sw3.up.is_high()),
            _ => return Err(HothouseError::BadIndex),
        };
        match sets {
            (false, true) => Ok(ToggleState::Down),
            (true, true) => Ok(ToggleState::Centered),
            (true, false) => Ok(ToggleState::Up),
            (false, false) => Err(HothouseError::SwitchFailure),
        }
    }

    pub fn set_led(&mut self, led_id: usize, state: bool) -> Result<(), HothouseError> {
        match (led_id, state) {
            (1, true) => self.leds.led1.set_high(),
            (2, true) => self.leds.led2.set_high(),
            (1, false) => self.leds.led1.set_low(),
            (2, false) => self.leds.led2.set_low(),
            _ => return Err(HothouseError::BadIndex),
        }
        Ok(())
    }

    pub fn read_footswitch(&mut self, fsw_id: usize) -> Result<bool, HothouseError> {
        match fsw_id {
            1 => Ok(self.footswitches.fsw1.is_low()),
            2 => Ok(self.footswitches.fsw2.is_low()),
            _ => Err(HothouseError::BadIndex),
        }
    }
}

