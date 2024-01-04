#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::borrow::BorrowMut;
use core::cell::RefCell;

use defmt::{info, println};
use defmt_rtt as _;

use embassy_executor::Spawner;
use embassy_stm32::gpio::{AnyPin, Input, Level, Output, Pull, Speed};
// use cortex_m_rt::entry;
use embassy_stm32::peripherals::{self, DMA2_CH2, DMA2_CH7, PC8, PC9, USART1};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::pubsub::{PubSubBehavior, PubSubChannel};
use embassy_time::{Instant, Timer};
use panic_probe as _;
// use panic_halt as _;
use embassy_stm32::usart::{Config, Uart};
// use embassy_stm32::dma::NoDma;
use embassy_stm32::{bind_interrupts, timer::Channel, usart};
// use embassy_time::Delay;
// use embedded_hal::blocking::delay::DelayMs;
use heapless::{String, Vec};
use simple_robot::chassis::{Chassis, WheelDrive, WheelPinPair};

#[derive(Clone)]
enum DirectionCommand {
    Forward,
    Back,
    Left,
    Right,
    Stop,
    Unknown,
}

#[derive(Clone, PartialEq)]
enum ControlMode {
    AutoPilot,
    Manual,
}

static SHARED: PubSubChannel<ThreadModeRawMutex, DirectionCommand, 1, 2, 2> = PubSubChannel::new();
static CONTROL_MODE: Mutex<ThreadModeRawMutex, RefCell<ControlMode>> =
    Mutex::new(RefCell::new(ControlMode::Manual));
static CURRENT_DIRECTION: Mutex<ThreadModeRawMutex, RefCell<DirectionCommand>> =
    Mutex::new(RefCell::new(DirectionCommand::Stop));
static DISTANCE: PubSubChannel<ThreadModeRawMutex, u64, 1, 2, 2> = PubSubChannel::new();

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<USART1>;
});

#[embassy_executor::task]
async fn wait_bluetooth_commands(mut usart: Uart<'static, USART1, DMA2_CH7, DMA2_CH2>) {
    info!("Start bluetooth");

    usart.write(b"Hello from Robot\r\n").await.unwrap();

    const COMMAND_LEN: usize = 10;
    let mut buf = [0_u8; 1];
    let mut msg_vec: Vec<u8, COMMAND_LEN> = Vec::new();

    loop {
        match usart.read(&mut buf).await {
            Ok(_) => {
                let last_buf_val = *buf.first().unwrap();
                let is_new_line_char = last_buf_val == 10 || last_buf_val == 13;
                let is_msg_vec_full = msg_vec.is_full();
                let is_new_line = is_msg_vec_full || is_new_line_char;

                if is_new_line {
                    let mut msg_vec_copy: Vec<u8, COMMAND_LEN> = Vec::new();

                    msg_vec_copy.clone_from(&msg_vec);
                    msg_vec.clear();

                    if is_msg_vec_full && !is_new_line_char {
                        msg_vec.extend_from_slice(&buf).unwrap();
                    }

                    info!("Value buf: {:?}", &buf);
                    let msg = String::from_utf8(msg_vec_copy).unwrap();
                    info!("Value str: {:?}", msg.as_str());

                    let command = match msg.as_str() {
                        "f" => DirectionCommand::Forward,
                        "b" => DirectionCommand::Back,
                        "l" => DirectionCommand::Left,
                        "r" => DirectionCommand::Right,
                        "s" => DirectionCommand::Stop,
                        _ => DirectionCommand::Unknown,
                    };

                    let control_mode = match msg.as_str() {
                        "a" => ControlMode::AutoPilot,
                        _ => ControlMode::Manual,
                    };

                    CONTROL_MODE.lock().await.borrow_mut().replace(control_mode);
                    SHARED.publish_immediate(command);
                } else {
                    msg_vec.extend_from_slice(&buf).unwrap();
                    info!("Value buf: {:?}", &buf);
                }

                info!("Vec value after all: {:?}", &msg_vec.as_slice());
            }
            Err(e) => info!("Error read: {:?}", e),
        };
    }
}

#[embassy_executor::task]
async fn handle_direction_command(mut chassis: Chassis<peripherals::PA0, peripherals::PA1>) {
    info!("Start command handler");

    let mut direction_sub = SHARED.subscriber().unwrap();

    loop {
        let direction = direction_sub.next_message_pure().await;

        match direction {
            DirectionCommand::Forward => chassis.forward(),
            DirectionCommand::Back => chassis.back(),
            DirectionCommand::Left => chassis.left(),
            DirectionCommand::Right => chassis.right(),
            DirectionCommand::Stop => chassis.stop(),
            _ => info!("Receive unknown command"),
        }
    }
}

#[embassy_executor::task]
async fn update_distance(sonar_output_pin: AnyPin, sonar_input_pin: AnyPin) {
    let mut sonar_trig = Output::new(sonar_output_pin, Level::Low, Speed::Low);
    let sonar_echo = Input::new(sonar_input_pin, Pull::None);

    loop {
        sonar_trig.set_low();
        Timer::after_micros(5).await;

        sonar_trig.set_high();
        Timer::after_micros(10).await;

        while !(sonar_echo.is_high()) {}

        let inst = Instant::now();

        while !(sonar_echo.is_low()) {}

        let duration = Instant::checked_duration_since(&Instant::now(), inst).unwrap();
        let distance_cm = duration.as_micros() / 2 / 29;

        info!("Distance cm: {:?}", distance_cm);

        DISTANCE.publish_immediate(distance_cm);

        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn handle_autopilot_mode() {
    let mut distance_sub = DISTANCE.subscriber().unwrap();

    loop {
        let distance_cm = distance_sub.next_message_pure().await;

        let control_mode = CONTROL_MODE.lock().await.borrow().clone();
        if control_mode != ControlMode::AutoPilot {
            continue;
        }

        let direction = CURRENT_DIRECTION.lock().await.borrow().clone();

        let new_direction = match direction {
            DirectionCommand::Forward if distance_cm < 50 => {
                info!("Should stop");
                DirectionCommand::Stop
            }
            DirectionCommand::Stop if distance_cm < 50 => {
                info!("Should rotate");
                if distance_cm % 2 == 0 {
                    DirectionCommand::Left
                } else {
                    DirectionCommand::Right
                }
            }
            DirectionCommand::Left | DirectionCommand::Right | DirectionCommand::Stop
                if distance_cm > 50 =>
            {
                info!("Should go forward");
                DirectionCommand::Forward
            }
            _ => DirectionCommand::Unknown,
        };

        match new_direction {
            DirectionCommand::Unknown => (),
            new_dir => {
                SHARED.publish_immediate(new_dir);
                Timer::after_secs(3).await;
            }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize and create handle for devicer peripherals
    let p = embassy_stm32::init(Default::default());
    println!("Hello from println!");

    let mut config = Config::default();
    config.baudrate = 9600;

    let usart =
        // Uart::new(p.USART6, p.PC7, p.PC6, Irqs, p.DMA1_CH6, p.DMA1_CH5, config).unwrap();
        Uart::new(p.USART1, p.PB7, p.PB6, Irqs, p.DMA2_CH7, p.DMA2_CH2, config).unwrap();

    let wheel_left = WheelPinPair::new(p.PA8, p.PA0);
    let wheel_right = WheelPinPair::new(p.PA7, p.PA1);
    let fwd = WheelDrive::new(wheel_left, wheel_right, Channel::Ch1);
    let chassis = Chassis::new(p.TIM1, fwd);

    // wheel_drive.set_polarity(Channel::Ch2, OutputPolarity::ActiveHigh);
    // wheel_drive.set_duty(Channel::Ch2, 0);
    // let mut left_rear_direction = Output::new(p.PA4, Level::High, Speed::High);
    // let mut right_rear_direction = Output::new(p.PC0, Level::Low, Speed::High);
    // wheel_drive.enable(Channel::Ch2);

    spawner.spawn(handle_direction_command(chassis)).unwrap();
    spawner.spawn(wait_bluetooth_commands(usart)).unwrap();
    spawner
        .spawn(update_distance(p.PC8.into(), p.PC9.into()))
        .unwrap();
    spawner.spawn(handle_autopilot_mode()).unwrap();

    let mut direction_sub = SHARED.subscriber().unwrap();
    // Application Loop
    loop {
        let direction = direction_sub.next_message_pure().await;
        let mut direction_lock = CURRENT_DIRECTION.lock().await;
        direction_lock.borrow_mut().replace(direction.clone());
        drop(direction_lock);

        match direction {
            DirectionCommand::Forward => info!("Command: Forward"),
            DirectionCommand::Back => info!("Command: Back"),
            DirectionCommand::Left => info!("Command: Left"),
            DirectionCommand::Right => info!("Command: Right"),
            DirectionCommand::Stop => info!("Command: Stop"),
            _ => info!("Receive unknown command"),
        }
    }
}
