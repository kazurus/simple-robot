use embassy_stm32::{
    gpio::{AnyPin, Level, Output, OutputType, Pin, Speed},
    peripherals::TIM1,
    time::khz,
    timer::{
        complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
        simple_pwm::PwmPin,
        Channel, Channel1ComplementaryPin, Channel1Pin, Channel2ComplementaryPin, Channel2Pin,
        OutputPolarity,
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

pub struct Chassis {
    pub fwd_ch: Channel,
    pub fwd_left_direction: Output<'static, AnyPin>,
    pub fwd_right_direction: Output<'static, AnyPin>,
    pub rwd_ch: Channel,
    pub rwd_left_direction: Output<'static, AnyPin>,
    pub rwd_right_direction: Output<'static, AnyPin>,
    pub chassis: ComplementaryPwm<'static, TIM1>,
}

impl Chassis {
    pub fn new<
        CH1: Channel1Pin<TIM1>,
        DP1: Peripheral<P = CH1> + 'static,
        CH1N: Channel1ComplementaryPin<TIM1>,
        DP1N: Peripheral<P = CH1N> + 'static,
        CH2: Channel2Pin<TIM1>,
        DP2: Peripheral<P = CH2> + 'static,
        CH2N: Channel2ComplementaryPin<TIM1>,
        DP2N: Peripheral<P = CH2N> + 'static,
    >(
        tim1: TIM1,
        fwd: WheelDrive<DP1, AnyPin, CH1, DP1N, AnyPin, CH1N>,
        rwd: WheelDrive<DP2, AnyPin, CH2, DP2N, AnyPin, CH2N>,
    ) -> Self {
        let ch1 = PwmPin::new_ch1(fwd.wheel_left.digital_pin, OutputType::PushPull);
        let ch1n = ComplementaryPwmPin::new_ch1(fwd.wheel_right.digital_pin, OutputType::PushPull);
        let ch2 = PwmPin::new_ch2(rwd.wheel_left.digital_pin, OutputType::PushPull);
        let ch2n = ComplementaryPwmPin::new_ch2(rwd.wheel_right.digital_pin, OutputType::PushPull);

        let mut chassis = ComplementaryPwm::new(
            tim1,
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

        let max_duty = chassis.get_max_duty();
        chassis.set_dead_time(max_duty / 1024);

        chassis.set_polarity(fwd.channel, OutputPolarity::ActiveHigh);
        chassis.set_duty(fwd.channel, 0);
        let fwd_left_direction = Output::new(fwd.wheel_left.analog_pin, Level::High, Speed::Low);
        let fwd_right_direction = Output::new(fwd.wheel_right.analog_pin, Level::Low, Speed::Low);

        chassis.set_polarity(rwd.channel, OutputPolarity::ActiveHigh);
        chassis.set_duty(rwd.channel, 0);
        let rwd_left_direction = Output::new(rwd.wheel_left.analog_pin, Level::High, Speed::Low);
        let rwd_right_direction = Output::new(rwd.wheel_right.analog_pin, Level::Low, Speed::Low);

        Self {
            fwd_ch: fwd.channel,
            fwd_left_direction,
            fwd_right_direction,
            rwd_ch: rwd.channel,
            rwd_left_direction,
            rwd_right_direction,
            chassis,
        }
    }

    pub fn start(&mut self) {
        let max_duty = self.chassis.get_max_duty();
        self.chassis.set_duty(self.fwd_ch, max_duty / 2);
        self.chassis.set_duty(self.rwd_ch, max_duty / 2);
        self.chassis.enable(self.fwd_ch);
        self.chassis.enable(self.rwd_ch);
    }

    pub fn stop(&mut self) {
        self.chassis.set_duty(self.fwd_ch, 0);
        self.chassis.set_duty(self.rwd_ch, 0);
        self.chassis.disable(self.fwd_ch);
        self.chassis.disable(self.rwd_ch);
    }

    pub fn forward(&mut self) {
        self.fwd_left_direction.set_high();
        self.fwd_right_direction.set_low();
        self.rwd_left_direction.set_high();
        self.rwd_right_direction.set_low();
        self.start();
    }

    pub fn back(&mut self) {
        self.fwd_left_direction.set_low();
        self.fwd_right_direction.set_high();
        self.rwd_left_direction.set_low();
        self.rwd_right_direction.set_high();
        self.start();
    }

    pub fn left(&mut self) {
        self.fwd_left_direction.set_low();
        self.fwd_right_direction.set_low();
        self.rwd_left_direction.set_low();
        self.rwd_right_direction.set_low();
        self.start();
    }

    pub fn right(&mut self) {
        self.fwd_left_direction.set_high();
        self.fwd_right_direction.set_high();
        self.rwd_left_direction.set_high();
        self.rwd_right_direction.set_high();
        self.start();
    }
}
