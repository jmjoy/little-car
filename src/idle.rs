use embassy_stm32::{
    gpio::{Level, Output, Pin, Speed},
    Peripheral,
};

pub struct Idle {
    out: Output<'static>,
    enable: bool,
}

impl Idle {
    pub fn new(pin: impl Peripheral<P = impl Pin> + 'static) -> Self {
        Self {
            out: Output::new(pin, Level::High, Speed::VeryHigh),
            enable: true,
        }
    }

    pub fn set_idle(&mut self, idle: bool) {
        if self.enable {
            self.out.set_level(idle.into());
        }
    }

    pub fn set_enable(&mut self, enable: bool) {
        self.enable = enable;
        self.out.set_level(enable.into());
    }
}
