#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, warn, println};
use defmt_rtt as _;
// use defmt_brtt as _;
use panic_probe as _;
// use panic_halt as _;
use cortex_m_rt::entry;
use embassy_stm32::timer::Channel;
use embassy_time::Delay;
use embedded_hal::blocking::delay::DelayMs;
use simple_robot::chassis::{Chassis, WheelComplementaryPinPair, WheelDrive, WheelPinPair};

#[entry]
fn main() -> ! {
    // Initialize and create handle for devicer peripherals
    let p = embassy_stm32::init(Default::default());
    println!("Hello from println!");
    info!("Check info");
    warn!("Check warn");

    let wheel_left = WheelPinPair::new(p.PA8, p.PA0);
    let wheel_right = WheelComplementaryPinPair::new(p.PA7, p.PA1);
    let fwd = WheelDrive::new(wheel_left, wheel_right, Channel::Ch1);
    let mut chassis = Chassis::new(p.TIM1, fwd);

    // wheel_drive.set_polarity(Channel::Ch2, OutputPolarity::ActiveHigh);
    // wheel_drive.set_duty(Channel::Ch2, 0);
    // let mut left_rear_direction = Output::new(p.PA4, Level::High, Speed::High);
    // let mut right_rear_direction = Output::new(p.PC0, Level::Low, Speed::High);
    // wheel_drive.enable(Channel::Ch2);

    // Application Loop
    loop {
        chassis.forward();
        Delay.delay_ms(5000_u32);

        chassis.back();
        Delay.delay_ms(5000_u32);

        chassis.stop();
        Delay.delay_ms(5000_u32);

        chassis.left();
        Delay.delay_ms(5000_u32);

        chassis.right();
        Delay.delay_ms(5000_u32);
    }
}
