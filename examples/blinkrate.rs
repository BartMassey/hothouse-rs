#![no_main]
#![no_std]

use hothouse::{Hothouse, ToggleState, hal::prelude::*};

use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut hh = Hothouse::take();

    let pots = [[4, 1], [5, 2], [6, 3]];
    let mut rate: Option<f32> = None;
    let mut lit = false;

    loop {
        if let Some(rate) = rate {
            let time = (1000.0 / rate) as u32;
            hh.delay.delay_ms(time);
            lit = !lit;
            hh.set_led(1, lit).unwrap();
        }
        rate = None;
        for (i, ps) in pots.iter().enumerate() {
            let t = hh.read_toggle(i).unwrap();
            let p = match t {
                ToggleState::Centered => continue,
                ToggleState::Down => ps[0],
                ToggleState::Up => ps[1],
            };
            let f = hh.read_knob(p).unwrap();
            rate = Some(30.0 * f);
        }
    }
}
