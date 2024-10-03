use crate::control::ControlMode;
use defmt::Format;
use embassy_stm32::{
    interrupt::typelevel::Binding,
    mode::Async,
    usart::{self, Config, RxDma, RxPin, UartRx},
    Peripheral,
};

#[derive(Format)]
pub enum BluetoothAction {
    Stop,
    Forward,
    Backward,
    TurnLeft,
    TurnRight,
    ForwardLeft,
    ForwardRight,
    BackwardLeft,
    BackwardRight,
    Speed(u8),
    ControlMode(ControlMode),
}

pub struct Bluetooth {
    rx: UartRx<'static, Async>,
}

impl Bluetooth {
    pub fn new<T: usart::Instance>(
        peri: impl Peripheral<P = T> + 'static,
        irq: impl Binding<T::Interrupt, usart::InterruptHandler<T>> + 'static,
        rx: impl Peripheral<P = impl RxPin<T>> + 'static,
        rx_dma: impl Peripheral<P = impl RxDma<T>> + 'static,
    ) -> Self {
        let mut config = <Config as Default>::default();
        config.baudrate = 9600;

        Self {
            rx: UartRx::new(peri, irq, rx, rx_dma, config).unwrap(),
        }
    }

    pub async fn receive(&mut self) -> Result<BluetoothAction, u8> {
        let mut buf = [0u8; 1];
        self.rx.read(&mut buf).await.unwrap();

        match buf[0] {
            0x30 => Ok(BluetoothAction::Stop),
            0x31 => Ok(BluetoothAction::Forward),
            0x32 => Ok(BluetoothAction::ForwardRight),
            0x33 => Ok(BluetoothAction::TurnRight),
            0x34 => Ok(BluetoothAction::BackwardRight),
            0x35 => Ok(BluetoothAction::Backward),
            0x36 => Ok(BluetoothAction::BackwardLeft),
            0x37 => Ok(BluetoothAction::TurnLeft),
            0x38 => Ok(BluetoothAction::ForwardLeft),
            0x40 => Ok(BluetoothAction::ControlMode(ControlMode::AutoTrack)),
            0x41 => Ok(BluetoothAction::ControlMode(ControlMode::Bluetooth)),
            s @ 0x50..0x59 => Ok(BluetoothAction::Speed(50 + (s - 0x50) * 5)),
            _ => Err(buf[0]),
        }
    }
}
