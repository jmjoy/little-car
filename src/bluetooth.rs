use crate::control::{CarControl, ControlMode};
use defmt::Format;
use embassy_stm32::{
    interrupt::typelevel::Binding,
    mode::Async,
    usart::{self, Config, RxDma, RxPin, UartRx},
    Peripheral,
};

#[derive(Format)]
pub enum BluetoothAction {
    Car(CarControl),
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
            0x30 => Ok(BluetoothAction::Car(CarControl::Stop)),
            0x31 => Ok(BluetoothAction::Car(CarControl::Forward)),
            0x32 => Ok(BluetoothAction::Car(CarControl::ForwardRight)),
            0x33 => Ok(BluetoothAction::Car(CarControl::TurnRight)),
            0x34 => Ok(BluetoothAction::Car(CarControl::BackwardRight)),
            0x35 => Ok(BluetoothAction::Car(CarControl::Backward)),
            0x36 => Ok(BluetoothAction::Car(CarControl::BackwardLeft)),
            0x37 => Ok(BluetoothAction::Car(CarControl::TurnLeft)),
            0x38 => Ok(BluetoothAction::Car(CarControl::ForwardLeft)),
            s @ 0x50..0x59 => Ok(BluetoothAction::Car(CarControl::SetSpeed(
                50 + (s - 0x50) * 5,
            ))),
            0x40 => Ok(BluetoothAction::ControlMode(ControlMode::AutoTrack)),
            0x41 => Ok(BluetoothAction::ControlMode(ControlMode::Bluetooth)),
            _ => Err(buf[0]),
        }
    }
}
