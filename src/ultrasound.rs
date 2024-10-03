use defmt::debug;
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Output, Pin, Pull, Speed},
    Peripheral,
};
use embassy_time::{Instant, Timer};

pub struct Ultrasound {
    trig: Output<'static>,
    echo: ExtiInput<'static>,
}

impl Ultrasound {
    pub fn new<T: Pin>(
        trig_pin: impl Peripheral<P = impl Pin> + 'static,
        echo_pin: impl Peripheral<P = T> + 'static,
        echo_ch: impl Peripheral<P = T::ExtiChannel> + 'static,
    ) -> Self {
        Self {
            trig: Output::new(trig_pin, false.into(), Speed::VeryHigh),
            echo: ExtiInput::new(echo_pin, echo_ch, Pull::None),
        }
    }

    pub async fn distance(&mut self) -> f64 {
        self.trig.set_high();
        Timer::after_micros(20).await;
        self.trig.set_low();

        self.echo.wait_for_high().await;
        let last_time = Instant::now();
        self.echo.wait_for_low().await;
        let dur = Instant::now() - last_time;

        let distance = dur.as_micros() as f64 * 0.034 / 2.;

        Timer::after_millis(100).await;

        debug!("distance: {}", distance);

        distance
    }
}
