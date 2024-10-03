use core::cmp::min;
use embassy_stm32::{
    gpio::{Level, Output, OutputType, Pin, Speed},
    time::Hertz,
    timer::{
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
        Channel2Pin, Channel3Pin, GeneralInstance4Channel,
    },
    Peripheral,
};

pub struct Car<T: GeneralInstance4Channel> {
    pwma: SimplePwmChannel<'static, T>,
    ain1: Output<'static>,
    ain2: Output<'static>,

    pwmb: SimplePwmChannel<'static, T>,
    bin1: Output<'static>,
    bin2: Output<'static>,

    speed: u8,
}

impl<T: GeneralInstance4Channel> Car<T> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'static,
        ch2_pin: impl Peripheral<P = impl Channel2Pin<T>> + 'static,
        ch3_pin: impl Peripheral<P = impl Channel3Pin<T>> + 'static,
        ain1_pin: impl Peripheral<P = impl Pin> + 'static,
        ain2_pin: impl Peripheral<P = impl Pin> + 'static,
        bin1_pin: impl Peripheral<P = impl Pin> + 'static,
        bin2_pin: impl Peripheral<P = impl Pin> + 'static,
    ) -> Self {
        let pwm = SimplePwm::new(
            tim,
            None,
            Some(PwmPin::new_ch2(ch2_pin, OutputType::PushPull)),
            Some(PwmPin::new_ch3(ch3_pin, OutputType::PushPull)),
            None,
            Hertz::khz(20),
            CountingMode::EdgeAlignedUp,
        );
        let channels = pwm.split();
        Self {
            pwma: channels.ch2,
            ain1: Output::new(ain1_pin, Level::Low, Speed::VeryHigh),
            ain2: Output::new(ain2_pin, Level::Low, Speed::VeryHigh),
            pwmb: channels.ch3,
            bin1: Output::new(bin1_pin, Level::Low, Speed::VeryHigh),
            bin2: Output::new(bin2_pin, Level::Low, Speed::VeryHigh),
            speed: 90,
        }
    }

    pub fn set_speed(&mut self, speed: u8) {
        self.speed = min(speed, 100);
    }

    pub fn stop(&mut self) {
        self.ain1.set_low();
        self.ain2.set_low();
        self.bin1.set_low();
        self.bin2.set_low();

        self.pwma.disable();
        self.pwmb.disable();
    }

    pub fn forward(&mut self) {
        self.ain1.set_low();
        self.ain2.set_high();
        self.bin1.set_low();
        self.bin2.set_high();

        self.pwma.set_duty_cycle_percent(self.speed);
        self.pwmb.set_duty_cycle_percent(self.speed);

        self.pwma.enable();
        self.pwmb.enable();
    }

    pub fn backward(&mut self) {
        self.ain1.set_high();
        self.ain2.set_low();
        self.bin1.set_high();
        self.bin2.set_low();

        self.pwma.set_duty_cycle_percent(self.speed);
        self.pwmb.set_duty_cycle_percent(self.speed);

        self.pwma.enable();
        self.pwmb.enable();
    }

    pub fn turn_left(&mut self) {
        self.ain1.set_high();
        self.ain2.set_low();
        self.bin1.set_low();
        self.bin2.set_high();

        self.pwma.set_duty_cycle_percent(self.speed);
        self.pwmb.set_duty_cycle_percent(self.speed);

        self.pwma.enable();
        self.pwmb.enable();
    }

    pub fn turn_right(&mut self) {
        self.ain1.set_low();
        self.ain2.set_high();
        self.bin1.set_high();
        self.bin2.set_low();

        self.pwma.set_duty_cycle_percent(self.speed);
        self.pwmb.set_duty_cycle_percent(self.speed);

        self.pwma.enable();
        self.pwmb.enable();
    }

    pub fn forward_left(&mut self) {
        self.ain1.set_low();
        self.ain2.set_high();
        self.bin1.set_low();
        self.bin2.set_high();

        self.pwma.set_duty_cycle_percent(self.speed / 2);
        self.pwmb.set_duty_cycle_percent(self.speed);

        self.pwma.enable();
        self.pwmb.enable();
    }

    pub fn forward_right(&mut self) {
        self.ain1.set_low();
        self.ain2.set_high();
        self.bin1.set_low();
        self.bin2.set_high();

        self.pwma.set_duty_cycle_percent(self.speed);
        self.pwmb.set_duty_cycle_percent(self.speed / 2);

        self.pwma.enable();
        self.pwmb.enable();
    }

    pub fn backward_left(&mut self) {
        self.ain1.set_high();
        self.ain2.set_low();
        self.bin1.set_high();
        self.bin2.set_low();

        self.pwma.set_duty_cycle_percent(self.speed / 2);
        self.pwmb.set_duty_cycle_percent(self.speed);

        self.pwma.enable();
        self.pwmb.enable();
    }

    pub fn backward_right(&mut self) {
        self.ain1.set_high();
        self.ain2.set_low();
        self.bin1.set_high();
        self.bin2.set_low();

        self.pwma.set_duty_cycle_percent(self.speed);
        self.pwmb.set_duty_cycle_percent(self.speed / 2);

        self.pwma.enable();
        self.pwmb.enable();
    }
}
