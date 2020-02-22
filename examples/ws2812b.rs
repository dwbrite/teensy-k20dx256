#![feature(stdsimd)]
#![no_std]
#![no_main]

use teensy::*;
use teensy::spi::Spi0;
use teensy::sleep::sleep_ms;

use embedded_hal::blocking::delay::DelayMs;

use ws2812_spi::Ws2812;
use smart_leds_trait::{RGB8, SmartLedsWrite};
use teensy::mcg::CpuFreq;

define_panic! {empty}

#[no_mangle]
fn main() {
    // set up clocks
    unsafe {
        let mcg = mcg::Mcg::new();
        let sim = sim::Sim::new();
        mcg.set_clocks(CpuFreq::High, sim);
    }

    // set up pins
    //?this might already be taken care of?

    // set up "delay" object (hmm)
    let mut delay = Delay{};

    // TODO: configure SPI
    let spi = unsafe {
        Spi0::new()
    };

    spi.start();
    spi.set_mode(ws2812_spi::MODE);
    spi.set_divider(2,8);

    let mut data: [RGB8; 3] = [RGB8::default(); 3];
    let empty: [RGB8; 3] = [RGB8::default(); 3];
    let mut ws = Ws2812::new(spi);
    loop {
        data[0] = RGB8 {
            r: 0,
            g: 0,
            b: 0x10,
        };
        data[1] = RGB8 {
            r: 0,
            g: 0x10,
            b: 0,
        };
        data[2] = RGB8 {
            r: 0x10,
            g: 0,
            b: 0,
        };

        ws.write(data.iter().cloned()).unwrap();
        delay.delay_ms(1000);
        ws.write(empty.iter().cloned()).unwrap();
        delay.delay_ms(1000);
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into()
    }
    if wheel_pos < 170 {
        wheel_pos -=85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into()
    }
    wheel_pos -= 170;
    (wheel_pos*3, 255 - wheel_pos * 3, 0).into()
}

struct Delay {}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        sleep_ms(ms.into())
    }
}