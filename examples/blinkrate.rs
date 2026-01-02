/*!

Adjust the blink rate of the left LED using the knobs and
toggle switches.

Basic operation: toggle a switch up for the knob in the top
bank or down for the knob in the bottom bank.  Adjust the
selected knob to adjust the blink rate of the left LED. The
switches are read with leftmost-priority: the first
non-centered switch is chosen.

The right LED stays on solid. Audio is passed through.
*/

#![no_main]
#![no_std]

use hothouse::{Hothouse, ToggleState, audio_passthrough, hal::prelude::*};

use cortex_m_rt::entry;
// Need a floating-point library for calculations below.
use num_traits::float::Float;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut hh = Hothouse::take(audio_passthrough);

    let pots = [[4, 1], [5, 2], [6, 3]];
    let mut rate = 0.0f32;
    let mut lit = false;

    loop {
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
    }
}
