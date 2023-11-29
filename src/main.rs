#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
// use stm32f4::stm32f401;
// use panic_halt as _;
use stm32f4xx_hal::{
    gpio::Pin,
    pac::{self},
    prelude::*,
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello World 1");

    let mut peripherals = pac::Peripherals::take().unwrap();


    let gpioa = peripherals.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output();

    let gpioc = peripherals.GPIOC.split();
    let mut button = gpioc.pc13;

    let mut delay_counter = 10_000_i32;

    led.set_low();

    loop {
        delay_counter = loop_delay(delay_counter, &button);
        
        led.toggle();
    }
}

fn loop_delay<const P: char, const N: u8>(mut del: i32, but: &Pin<P, N>) -> i32 {
    for _ in 1..del {
        if but.is_low() {
            rprintln!("Button got pressed");

            del = del - 2_5000_i32;

            if del < 2_5000 {
                del = 10_0000_i32;
            }

            return del
        }
    } 

    del
}
