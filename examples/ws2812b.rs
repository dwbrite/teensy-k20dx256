#![feature(stdsimd)]
#![no_std]
#![no_main]

use teensy::*;
use teensy::spi::Spi0;
use teensy::sleep::sleep_ms;

use embedded_hal::blocking::delay::DelayMs;

use ws2812_spi::{Ws2812, prerendered};
use smart_leds_trait::{RGB8, SmartLedsWrite};
use teensy::mcg::CpuFreq;
use ws2812_spi::prerendered::Timing;
use smart_leds::brightness;

define_panic! {blink}




#[no_mangle]
fn main() {
    // set up clocks
    unsafe {
        let mcg = mcg::Mcg::new();
        let sim = sim::Sim::new();
        mcg.set_clocks(CpuFreq::High, sim);
    }

    // set up pins
    //   pins are set up in spi.start()

    // set up "delay" object (hmm)
    let mut delay = Delay{};

    // configure SPI
    let spi = unsafe {
        let spi = Spi0::new();
        spi.start();
        spi.set_mode(ws2812_spi::MODE);
        spi.set_divider(2,8);
        spi
    };

    //pre_rainbow(spi, &mut delay);
    spi_blink(spi, &mut delay);

}

fn spi_blink(spi: &mut Spi0, delay: &mut Delay) {

    const L: usize = 30;
    if L % 3 != 0 {
        panic!("must be divisible by 3")
    }

    let mut data: [RGB8; L] = [RGB8::default(); L];
    let empty: [RGB8; L] = [RGB8::default(); L];
    let mut ws = Ws2812::new(spi);
    loop {
        for i in 0..L/3 {
            data[i*3+0] = RGB8 {
                r: 0,
                g: 0,
                b: 0x10,
            };
            data[i*3+1] = RGB8 {
                r: 0,
                g: 0x10,
                b: 0,
            };
            data[i*3+2] = RGB8 {
                r: 0x10,
                g: 0,
                b: 0,
            };
        }
        ws.write(data.iter().cloned()).unwrap();
        delay.delay_ms(1000 as u16);
        ws.write(empty.iter().cloned()).unwrap();
        delay.delay_ms(1000 as u16);
    }
}

fn pre_rainbow(spi: &mut Spi0, delay: &mut Delay) {
    const NUM_LEDS: usize = 30;
    let mut data = [RGB8::default().into(); NUM_LEDS];
    let mut rendered_data = [0; NUM_LEDS * 3 * 5];

    let mut neopixel = prerendered::Ws2812::new(spi, Timing::new(4_000_000).unwrap(), &mut rendered_data);

    loop {
        for j in 0..(256*5) {
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
            neopixel.write(brightness(data.iter().cloned(), 32)).unwrap();
            delay.delay_ms(10);
        }
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

fn blink_short() {
    let mut led = unsafe { make_pin!(led).make_gpio().with_output() };

    led.high();
    sleep_ms(200);
    led.low();
    sleep_ms(1000);
}

fn double_blink() {
    let mut led = unsafe { make_pin!(led).make_gpio().with_output() };

    led.high();
    sleep_ms(200);
    led.low();

    sleep_ms(200);

    led.high();
    sleep_ms(200);
    led.low();

    sleep_ms(1000);
}