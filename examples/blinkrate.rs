#![no_main]
#![no_std]

use hothouse::{Hothouse, ToggleState, hal::prelude::*};

use cortex_m_rt::entry;
use num_traits::float::Float;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut hh = Hothouse::take();
    hh.set_led(2, true).unwrap();

    let pots = [[4, 1], [5, 2], [6, 3]];
    let mut rate = 0.0f32;
    let mut lit = false;

    loop {
        let time = (300.0 / 10.0.powf(rate)).floor() as u32;
        hh.delay.delay_ms(time);
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
