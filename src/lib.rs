/*!

Cleveland Music Hothouse pedal support crate. This crate
wraps the [`daisy`](https://crates.io/crates/daisy) crate to
provide Hothouse-specific support and convenience functions.

*/

#![no_main]
#![no_std]

use core::cell::RefCell;

/// Underlying processor support crate.
pub use cortex_m;
/// Underlying `daisy` crate modules.
pub use daisy::{self as board, hal, pac};
pub use daisy::audio::{
    /// Length of audio block in frames.
    BLOCK_LENGTH,
    /// Frame rate in frames per second.
    FS,
}};

use cortex_m::interrupt::Mutex;
use daisy::{
    audio,
    hal::{prelude::*, adc, delay, gpio, rcc},
    pac::interrupt,
    Board,
};

static AUDIO_INTERFACE: Mutex<RefCell<Option<AudioState>>> =
    Mutex::new(RefCell::new(None));

/// Errors resulting from Hothouse things.
#[derive(Debug)]
pub enum HothouseError {
    /// Index for operation was out of range. Note that Hothouse
    /// indexes are 1-based.
    BadIndex,
    /// A Hothouse toggle switch was found in an impossible state.
    SwitchFailure,
}

/// Left and right audio channels.
pub type AudioFrame = (f32, f32);

/// An audio handler takes a tick count in sample blocks and modifies
/// the given block as desired to produce effected audio.
pub type AudioHandler = fn(u64, &mut [AudioFrame; BLOCK_LENGTH]);

struct AudioState {
    tick: u64,
    audio: audio::Interface,
    handler: AudioHandler,
}

/// Per-knob GPIO assignments. These are all set up as
/// analog inputs.
pub struct Knobs {
    pub knob1: gpio::gpioa::PA3<gpio::Analog>,
    pub knob2: gpio::gpiob::PB1<gpio::Analog>,
    pub knob3: gpio::gpioa::PA7<gpio::Analog>,
    pub knob4: gpio::gpioa::PA6<gpio::Analog>,
    pub knob5: gpio::gpioc::PC1<gpio::Analog>,
    pub knob6: gpio::gpioc::PC4<gpio::Analog>,
}

/// A pair of GPIOs for a single-pole double-throw toggle
/// switch. These GPIOs are set up as pull-up inputs.
pub struct Spdt<Down, Up> {
    pub down: Down,
    pub up: Up,
}

/// The GPIOs for the three toggle switches.
pub struct Toggles {
    pub sw1: Spdt<gpio::gpiob::PB5<gpio::Input>, gpio::gpiob::PB4<gpio::Input>>,
    pub sw2: Spdt<gpio::gpiog::PG11<gpio::Input>, gpio::gpiog::PG10<gpio::Input>>,
    pub sw3: Spdt<gpio::gpioc::PC12<gpio::Input>, gpio::gpiod::PD2<gpio::Input>>,
}

/// Representation of the state of a single-pole
/// double-throw toggle switch.
pub enum ToggleState {
    Down,
    Centered,
    Up,
}

/// GPIO assignments for the LEDs. These GPIOs are set up as
/// push-pull outputs.
pub struct Leds {
    pub led1: gpio::gpioa::PA5<gpio::Output>,
    pub led2: gpio::gpioa::PA4<gpio::Output>,
}

/// GPIO assignments for the momentary contact footswitches.
/// These GPIOs are set up as pull-up inputs.
pub struct Footswitches {
    pub fsw1: gpio::gpioa::PA0<gpio::Input>,
    pub fsw2: gpio::gpiod::PD11<gpio::Input>,
}

/// The ongoing state of the Hothouse software.
pub struct Hothouse {
    /// The `daisy::Board` struct.
    pub board: Board,
    /// A preconfigured delay in case it is useful.
    pub delay: hal::delay::Delay,
    /// System clocks for making new peripherals.
    pub clocks: rcc::CoreClocks,
    /// The ADC set up to read the knobs.
    pub knob_adc: adc::Adc<pac::ADC1, adc::Enabled>,
    /// The knobs.
    pub knobs: Knobs,
    /// The LEDs.
    pub leds: Leds,
    /// The toggle switches.
    pub toggles: Toggles,
    /// The foot switches.
    pub footswitches: Footswitches,
}

/// Pure pass-through convenience handler for audio.
pub fn audio_passthrough(_tick: u64, _frames: &mut [AudioFrame; BLOCK_LENGTH]) {}

impl Hothouse {
    /// Initialize the Hothouse and return the state
    /// structure.  This also initializes internal global
    /// state for the audio interrupt handler.
    ///
    /// An audio handler must be provided: use
    /// [audio_passthrough] as needed.
    pub fn take(handler: AudioHandler) -> Self {
        let mut cp = cortex_m::Peripherals::take().unwrap();
        let dp = pac::Peripherals::take().unwrap();
        let board = Board::take().unwrap();

        // Using caches should provide a major performance boost.
        cp.SCB.enable_icache();
        // NOTE: Data caching requires cache management around all use of DMA.
        // The `daisy` crate already handles that for audio processing.
        cp.SCB.enable_dcache(&mut cp.CPUID);

        let ccdr = daisy::board_freeze_clocks!(board, dp);
        let pins = daisy::board_split_gpios!(board, ccdr, dp);

        let audio = daisy::board_split_audio!(ccdr, pins);
        let audio = audio.spawn().unwrap();
        let audio_state = AudioState { audio, tick: 0, handler };
        cortex_m::interrupt::free(|cs| {
            *AUDIO_INTERFACE.borrow(cs).borrow_mut() = Some(audio_state);
        });

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

        Self { board, delay, clocks: ccdr.clocks, knob_adc, knobs, toggles, leds, footswitches }
    }

    /// Get a value between 0.0 and 1.0 for the specified
    /// knob.  Knob ids are 1-based.
    ///
    /// # Errors
    ///
    /// Returns [HothouseError::BadIndex] if the `knob_id` is
    /// out of range.
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

    /// Get the position of the specified toggle switch.
    /// Switch ids are 1-based.
    ///
    /// # Errors
    ///
    /// Returns [HothouseError::BadIndex] if the `toggle_id`
    /// is out of range. Returns
    /// [HothouseError::SwitchFailure] if the switch appears
    /// to be simultaneously in the up and down positions.
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

    /// Set the specified LED to the specified state.  LED
    /// ids are 1-based.
    ///
    /// # Errors
    ///
    /// Returns [HothouseError::BadIndex] if the `led_id` is
    /// out of range.
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

    /// Get the position of the specified footswitch.
    /// Footswitch ids are 1-based.
    ///
    /// # Errors
    ///
    /// Returns [HothouseError::BadIndex] if the `fsw_id`
    /// is out of range.
    pub fn read_footswitch(&mut self, fsw_id: usize) -> Result<bool, HothouseError> {
        match fsw_id {
            1 => Ok(self.footswitches.fsw1.is_low()),
            2 => Ok(self.footswitches.fsw2.is_low()),
            _ => Err(HothouseError::BadIndex),
        }
    }
}

// Audio is tranfered from the input and to the input
// periodically through DMA.  Every time Daisy is done
// transferring data, it will ask for more by triggering the
// DMA 1 Stream 1 interrupt.
#[interrupt]
fn DMA1_STR1() {
    cortex_m::interrupt::free(|cs| {
        // Acquire the audio interface from the global.
        if let Some(audio_state) = AUDIO_INTERFACE.borrow(cs).borrow_mut().as_mut() {
            // Read input audio from the buffer and write back desired
            // output samples.
            audio_state.audio.handle_interrupt_dma1_str1(|audio_buffer| {
                (audio_state.handler)(audio_state.tick, audio_buffer);
                audio_state.tick += 1;
            }).unwrap();
        }
    });
}
