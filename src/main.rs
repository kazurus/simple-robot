#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use stm32f4xx_hal::{
    pac::{self},
    prelude::*,
    timer::{Channel, Channel3},
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Start STM programm");

    let mut peripherals = pac::Peripherals::take().unwrap();

    let rcc = peripherals.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();

    let gpiob = peripherals.GPIOB.split();
    let engine = Channel3::new(gpiob.pb10.into_alternate());
    let mut engine_pwm = peripherals.TIM2.pwm_hz(engine, 2000.Hz(), &clocks);

    let max_duty = engine_pwm.get_max_duty();
    engine_pwm.set_duty(Channel::C3, max_duty / 2);

    let mut delay = peripherals.TIM3.delay_ms(&clocks);

    let gpioa = peripherals.GPIOA.split();
    let mut engine_a = gpioa.pa0.into_push_pull_output();

    loop {
        engine_a.set_high();

        engine_pwm.set_period(500.Hz());
        engine_pwm.enable(Channel::C3);

        let temp = engine_pwm.get_period();
        rprintln!("{:?}", temp);

        delay.delay_ms(10000_u32);
        engine_pwm.disable(Channel::C3);
        delay.delay_ms(10000_u32);
    }
}
