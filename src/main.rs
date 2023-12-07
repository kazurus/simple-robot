#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, warn};
use defmt_rtt as _;
// use defmt_brtt as _;
use panic_probe as _;
// use panic_halt as _;
use cortex_m_rt::entry;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{AnyPin, Input, Level, Output, OutputType, Pin, Pull, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::{Channel, OutputPolarity};
use embassy_stm32::timer::complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin};
use embassy_stm32::timer::simple_pwm::PwmPin;
use embassy_time::{Delay, Duration, Timer};
use embedded_hal::blocking::delay::DelayMs;

#[entry]
fn main() -> ! {
    // Initialize and create handle for devicer peripherals
    let mut p = embassy_stm32::init(Default::default());
    defmt::println!("Hello Error!");

    // Configure the Buzzer pin as an alternate and obtain handler.
    // I will use PA9 that connects to Grove shield connector D8
    // On the Nucleo FR401 PA9 connects to timer TIM1
    let ch1 = PwmPin::new_ch1(p.PA8, OutputType::PushPull);
    let ch1n = ComplementaryPwmPin::new_ch1(p.PA7, OutputType::PushPull);
    let ch2 = PwmPin::new_ch2(p.PA9, OutputType::PushPull);
    let ch2n = ComplementaryPwmPin::new_ch2(p.PB0, OutputType::PushPull);

    let mut wheel_drive = ComplementaryPwm::new(
        p.TIM1,
        Some(ch1),
        Some(ch1n),
        Some(ch2),
        Some(ch2n),
        None,
        None,
        None,
        None,
        khz(2),
        Default::default(),
    );

    let max_duty = wheel_drive.get_max_duty();
    wheel_drive.set_dead_time(max_duty / 1024);

    wheel_drive.set_polarity(Channel::Ch1, OutputPolarity::ActiveHigh);
    wheel_drive.set_duty(Channel::Ch1, 0);
    let mut left_front_direction = Output::new(p.PA0, Level::High, Speed::High);
    let mut right_front_direction = Output::new(p.PA1, Level::Low, Speed::High);
    wheel_drive.enable(Channel::Ch1);

    wheel_drive.set_polarity(Channel::Ch2, OutputPolarity::ActiveHigh);
    wheel_drive.set_duty(Channel::Ch2, 0);
    let mut left_rear_direction = Output::new(p.PA4, Level::High, Speed::High);
    let mut right_rear_direction = Output::new(p.PC0, Level::Low, Speed::High);
    wheel_drive.enable(Channel::Ch2);

    // Application Loop
    loop {
        wheel_drive.set_duty(Channel::Ch1, max_duty / 4);
        wheel_drive.set_duty(Channel::Ch2, max_duty / 4);
        Delay.delay_ms(5000_u32);

        wheel_drive.set_duty(Channel::Ch1, max_duty / 2);
        wheel_drive.set_duty(Channel::Ch2, max_duty / 2);
        Delay.delay_ms(5000_u32);
    }
}
