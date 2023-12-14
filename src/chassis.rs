use core::marker::PhantomData;

use embassy_stm32::{
    gpio::{Level, Output, OutputType, Pin, Speed},
    peripherals::TIM1,
    time::khz,
    timer::{
        complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
        simple_pwm::PwmPin,
        CaptureCompare16bitInstance, Channel, Channel1ComplementaryPin, Channel1Pin,
        ComplementaryCaptureCompare16bitInstance, OutputPolarity,
    },
    Peripheral,
};

pub struct WheelPinPair<DP, AP, I, CH1>
where
    I: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
    CH1: Channel1Pin<I>,
    DP: Peripheral<P = CH1>,
    AP: Pin,
{
    pub digital_pin: DP,
    pub analog_pin: AP,
    p1: PhantomData<I>,
}

impl<DP, AP, I, CH1> WheelPinPair<DP, AP, I, CH1>
where
    I: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
    CH1: Channel1Pin<I>,
    DP: Peripheral<P = CH1>,
    AP: Pin,
{
    pub fn new(digital_pin: DP, analog_pin: AP) -> Self {
        Self {
            digital_pin,
            analog_pin,
            p1: PhantomData,
        }
    }
}

pub struct WheelComplementaryPinPair<DP, AP, I, CH1C>
where
    I: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
    CH1C: Channel1ComplementaryPin<I>,
    DP: Peripheral<P = CH1C>,
    AP: Pin,
{
    pub digital_pin: DP,
    pub analog_pin: AP,
    p1: PhantomData<I>,
}

impl<DP, AP, I, CH1C> WheelComplementaryPinPair<DP, AP, I, CH1C>
where
    I: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
    CH1C: Channel1ComplementaryPin<I>,
    DP: Peripheral<P = CH1C>,
    AP: Pin,
{
    pub fn new(digital_pin: DP, analog_pin: AP) -> Self {
        Self {
            digital_pin,
            analog_pin,
            p1: PhantomData,
        }
    }
}

pub struct WheelDrive<K, C, T, U, L, H, Z, X>
where
    L: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
    H: Channel1Pin<L>,
    K: Peripheral<P = H>,
    C: Pin,
    Z: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
    X: Channel1ComplementaryPin<Z>,
    T: Peripheral<P = X>,
    U: Pin,
{
    pub wheel_left: WheelPinPair<K, C, L, H>,
    pub wheel_right: WheelComplementaryPinPair<T, U, Z, X>,
    pub channel: Channel,
    // pub wheel_left_direction: Output<'a, C>,
    // pub wheel_right_direction: Output<'a, U>,
}

impl<K, C, T, U, L, H, Z, X> WheelDrive<K, C, T, U, L, H, Z, X>
where
    L: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
    H: Channel1Pin<L>,
    K: Peripheral<P = H>,
    C: Pin,
    Z: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
    X: Channel1ComplementaryPin<Z>,
    T: Peripheral<P = X>,
    U: Pin,
{
    pub fn new(
        wheel_left: WheelPinPair<K, C, L, H>,
        wheel_right: WheelComplementaryPinPair<T, U, Z, X>,
        channel: Channel,
    ) -> Self {
        Self {
            wheel_left,
            wheel_right,
            channel,
        }
    }
}

pub struct Chassis<'a, C, U>
where
    C: Pin,
    U: Pin,
{
    pub fwd_ch: Channel,
    pub fwd_left_direction: Output<'a, C>,
    pub fwd_right_direction: Output<'a, U>,
    pub chassis: ComplementaryPwm<'a, TIM1>,
}

impl<'a, C, U> Chassis<'a, C, U>
where
    C: Pin,
    U: Pin,
{
    pub fn new<
        L: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
        H: Channel1Pin<L> + Channel1Pin<TIM1>,
        K: Peripheral<P = H> + 'a,
        Z: ComplementaryCaptureCompare16bitInstance + CaptureCompare16bitInstance,
        X: Channel1ComplementaryPin<Z> + Channel1ComplementaryPin<TIM1>,
        T: Peripheral<P = X> + 'a,
    >(
        tim1: TIM1,
        fwd: WheelDrive<K, C, T, U, L, H, Z, X>,
    ) -> Self {
        let ch1 = PwmPin::<TIM1, _>::new_ch1(fwd.wheel_left.digital_pin, OutputType::PushPull);
        let ch1n = ComplementaryPwmPin::<TIM1, _>::new_ch1(
            fwd.wheel_right.digital_pin,
            OutputType::PushPull,
        );

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
        chassis.enable(fwd.channel);

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
