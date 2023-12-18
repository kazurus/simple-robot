use embassy_stm32::{
    gpio::{Level, Output, OutputType, Pin, Speed},
    peripherals::TIM1,
    time::khz,
    timer::{
        complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
        simple_pwm::PwmPin,
        Channel, Channel1ComplementaryPin, Channel1Pin, OutputPolarity,
    },
    Peripheral,
};

pub struct WheelPinPair<DP, AP, CH>
where
    DP: Peripheral<P = CH>,
    AP: Pin,
{
    pub digital_pin: DP,
    pub analog_pin: AP,
}

impl<DP, AP, CH> WheelPinPair<DP, AP, CH>
where
    DP: Peripheral<P = CH>,
    AP: Pin,
{
    pub fn new(digital_pin: DP, analog_pin: AP) -> Self {
        Self {
            digital_pin,
            analog_pin,
        }
    }
}

pub struct WheelDrive<DP1, AP1, CH1, DP2, AP2, CH2>
where
    DP1: Peripheral<P = CH1>,
    AP1: Pin,
    CH1: Pin,
    DP2: Peripheral<P = CH2>,
    AP2: Pin,
    CH2: Pin,
{
    pub wheel_left: WheelPinPair<DP1, AP1, CH1>,
    pub wheel_right: WheelPinPair<DP2, AP2, CH2>,
    pub channel: Channel,
}

impl<DP1, AP1, CH1, DP2, AP2, CH2> WheelDrive<DP1, AP1, CH1, DP2, AP2, CH2>
where
    DP1: Peripheral<P = CH1>,
    AP1: Pin,
    CH1: Pin,
    DP2: Peripheral<P = CH2>,
    AP2: Pin,
    CH2: Pin,
{
    pub fn new(
        wheel_left: WheelPinPair<DP1, AP1, CH1>,
        wheel_right: WheelPinPair<DP2, AP2, CH2>,
        channel: Channel,
    ) -> Self {
        Self {
            wheel_left,
            wheel_right,
            channel,
        }
    }
}

pub struct Chassis<AP1, AP2>
where
    AP1: Pin,
    AP2: Pin,
{
    pub fwd_ch: Channel,
    pub fwd_left_direction: Output<'static, AP1>,
    pub fwd_right_direction: Output<'static, AP2>,
    pub chassis: ComplementaryPwm<'static, TIM1>,
}

impl<AP1, AP2> Chassis<AP1, AP2>
where
    AP1: Pin,
    AP2: Pin,
{
    pub fn new<
        DP1: Peripheral<P = CH1> + 'static,
        CH1: Channel1Pin<TIM1>,
        DP2: Peripheral<P = CH2> + 'static,
        CH2: Channel1ComplementaryPin<TIM1>,
    >(
        tim1: TIM1,
        fwd: WheelDrive<DP1, AP1, CH1, DP2, AP2, CH2>,
    ) -> Self {
        let ch1 = PwmPin::new_ch1(fwd.wheel_left.digital_pin, OutputType::PushPull);
        let ch1n = ComplementaryPwmPin::new_ch1(fwd.wheel_right.digital_pin, OutputType::PushPull);

        let mut chassis = ComplementaryPwm::new(
            tim1,
            Some(ch1),
            Some(ch1n),
            None,
            None,
            // Some(ch2),
            // Some(ch2n),
            None,
            None,
            None,
            None,
            khz(2),
            Default::default(),
        );

        let max_duty = chassis.get_max_duty();
        chassis.set_dead_time(max_duty / 1024);

        chassis.set_polarity(fwd.channel, OutputPolarity::ActiveHigh);
        chassis.set_duty(fwd.channel, 0);
        let fwd_left_direction = Output::new(fwd.wheel_left.analog_pin, Level::High, Speed::Low);
        let fwd_right_direction = Output::new(fwd.wheel_right.analog_pin, Level::Low, Speed::Low);

        // let ch2 = PwmPin::new_ch2(p.PA9, OutputType::PushPull);
        // let ch2n = ComplementaryPwmPin::new_ch2(p.PB0, OutputType::PushPull);

        // wheel_drive.set_polarity(Channel::Ch2, OutputPolarity::ActiveHigh);
        // wheel_drive.set_duty(Channel::Ch2, 0);
        // let mut left_rear_direction = Output::new(p.PA4, Level::High, Speed::High);
        // let mut right_rear_direction = Output::new(p.PC0, Level::Low, Speed::High);
        // wheel_drive.enable(Channel::Ch2);
        //
        Self {
            fwd_ch: fwd.channel,
            fwd_left_direction,
            fwd_right_direction,
            chassis,
        }
    }

    pub fn start(&mut self) {
        let max_duty = self.chassis.get_max_duty();
        self.chassis.set_duty(self.fwd_ch, max_duty / 2);
        self.chassis.enable(self.fwd_ch);
    }

    pub fn stop(&mut self) {
        self.chassis.set_duty(self.fwd_ch, 0);
        self.chassis.disable(self.fwd_ch);
    }

    pub fn forward(&mut self) {
        self.fwd_left_direction.set_high();
        self.fwd_right_direction.set_low();
        self.start();
    }

    pub fn back(&mut self) {
        self.fwd_left_direction.set_low();
        self.fwd_right_direction.set_high();
        self.start();
    }

    pub fn left(&mut self) {
        self.fwd_left_direction.set_low();
        self.fwd_right_direction.set_low();
        self.start();
    }

    pub fn right(&mut self) {
        self.fwd_left_direction.set_high();
        self.fwd_right_direction.set_high();
        self.start();
    }
}
