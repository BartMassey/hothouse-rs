/*!

Mono tremolo with three independent units. Knob 1 is depth
and Knob 4 is rate for first tremolo unit; Toggle Switch 1
is up for sine, down for triangle, center for off. Two other
tremolo units in the same style. Footswitch 2 toggles
pass-through. LED 2 indicates active.

*/

#![no_main]
#![no_std]

use core::cell::RefCell;

use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
// Need a floating-point library for calculations below.
// use num_traits::float::Float;
use panic_halt as _;

use hothouse::{Hothouse, ToggleState, hal::prelude::*};

#[derive(Default)]
enum WaveShape {
    Sine,
    Triangle,
}

#[derive(Default)]
struct Tremolo {
    depth: f32,
    rate: f32,
    wave: Option<WaveShape>,
}

#[derive(Default)]
struct Params {
   channels: [Tremolo; 3],
}

static PARAMS: Mutex<RefCell<Params>> =
    Mutex::new(RefCell::new(Params::default()));

pub fn trembulous(_tick: u64, _frames: &mut [AudioFrame; BLOCK_LENGTH]) {
    todo!()
}

#[entry]
fn main() -> ! {
    let mut hh = Hothouse::take();

    let pots = [[4, 1], [5, 2], [6, 3]];
    let mut passthrough = true;
    let mut passthrough_held = hh.read_footswitch(2).unwrap();

    loop {
        // UI 100 ticks/sec.
        hh.delay.delay_ms(10u32);

        let fsw2 = hh.read_footswitch(2).unwrap();
        if !passthrough_held && fsw2 {
            passthrough_held = true;
            passthrough = !passthrough;
        } else if passthrough_held && !fsw2 {
            passthrough_held = false;
        }
        hh.set_led(2, !passthrough);

        /*
        hh.set_led(2, true).unwrap();
        let time = (300.0 / 10.0.powf(rate)).floor() as u32;
        hh.delay.delay_ms(time);
        hh.set_led(2, false).unwrap();
        lit = !lit;
        hh.set_led(1, lit).unwrap();

        rate = 0.0;
        for (i, ps) in pots.iter().enumerate() {
            let t = hh.read_toggle(i + 1).unwrap();
            let p = match t {
                ToggleState::Centered => continue,
                ToggleState::Down => ps[0],
                ToggleState::Up => ps[1],
            };
            rate = hh.read_knob(p).unwrap();
            break;
        }
        */
    }
}
