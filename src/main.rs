#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use stm32f4xx_hal::{
    pac::{self},
    prelude::*,
    timer::{Channel, Channel1, Channel2, Channel3, Polarity},
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Start STM programm");

    let peripherals = pac::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();

    let gpioa = peripherals.GPIOA.split();
    let gpiob = peripherals.GPIOB.split();
    let gpioc = peripherals.GPIOC.split();

    let engine = (
        Channel1::new(gpioa.pa8).with_complementary(gpioa.pa7),
        Channel2::new(gpioa.pa9).with_complementary(gpiob.pb0),
    );

    let mut engine = peripherals
        .TIM1
        .pwm_hz(engine, 2000.Hz(), &clocks)
        .split();

    let (mut engine_left, mut engine_right) = engine;

    let max_duty = engine_left.get_max_duty();
    engine_left.set_duty(max_duty / 2);
    engine_left.set_polarity(Polarity::ActiveHigh);
    engine_left.set_complementary_polarity(Polarity::ActiveHigh);

    let mut engine_left_front = gpioa.pa0.into_push_pull_output();
    let mut engine_left_rear = gpioa.pa1.into_push_pull_output();

    let max_duty = engine_right.get_max_duty();
    engine_right.set_duty(max_duty / 2);
    engine_right.set_polarity(Polarity::ActiveHigh);
    engine_right.set_complementary_polarity(Polarity::ActiveHigh);

    let mut engine_right_front = gpioa.pa4.into_push_pull_output();
    let mut engine_right_rear = gpioc.pc0.into_push_pull_output();

    let mut delay = peripherals.TIM3.delay_ms(&clocks);

    loop {
        engine_left_front.set_high();
        engine_left_rear.set_low();

        engine_right_front.set_high();
        engine_right_rear.set_low();
        
        engine_left.set_duty(max_duty / 2);
        engine_left.enable();
        engine_left.enable_complementary();

        engine_right.set_duty(max_duty / 2);
        engine_right.enable();
        engine_right.enable_complementary();
    }
}
