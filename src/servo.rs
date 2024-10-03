use embassy_stm32::{
    gpio::OutputType,
    time::Hertz,
    timer::{
        simple_pwm::{PwmPin, SimplePwm, SimplePwmChannel},
        Channel4Pin, GeneralInstance4Channel,
    },
    Peripheral,
};

pub struct Servo<T: GeneralInstance4Channel> {
    channel: SimplePwmChannel<'static, T>,
}

impl<T: GeneralInstance4Channel> Servo<T> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'static,
        pin: impl Peripheral<P = impl Channel4Pin<T>> + 'static,
    ) -> Self {
        let pwm = SimplePwm::new(
            tim,
            None,
            None,
            None,
            Some(PwmPin::new_ch4(pin, OutputType::PushPull)),
            Hertz::hz(50),
            embassy_stm32::timer::low_level::CountingMode::EdgeAlignedUp,
        );
        Self {
            channel: pwm.split().ch4,
        }
    }

    pub fn set_angle(&mut self, angle: u8) {
        self.channel.set_duty_cycle(
            ((self.channel.max_duty_cycle() as u64) * ((angle as u64) + 45) / 1800) as u16,
        );
        self.channel.enable();
    }
}
