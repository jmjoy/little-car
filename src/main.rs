#![no_std]
#![no_main]

mod bluetooth;
mod car;
mod control;
mod idle;
mod servo;
mod ultrasound;

use bluetooth::{Bluetooth, BluetoothAction};
use car::Car;
use control::{Control, ControlChannel, ControlMode, ControlModeWatch};
use core::future;
use defmt::{debug, error, info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_stm32::{
    bind_interrupts, pac, peripherals,
    rcc::{
        AHBPrescaler, APBPrescaler, Hse, HseMode, LsConfig, Pll, PllMul, PllPreDiv, PllSource,
        Sysclk,
    },
    time::Hertz,
    usart, Config,
};
use embassy_time::Timer;
use idle::Idle;
use panic_probe as _;
use servo::Servo;
use ultrasound::Ultrasound;

static CONTROL_CHANNEL: ControlChannel = ControlChannel::new();

static CONTROL_MODE_WATCH: ControlModeWatch = ControlModeWatch::new();

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

fn make_config() -> Config {
    let mut config = Config::default();
    config.rcc.hsi = true;
    config.rcc.hse = Some(Hse {
        freq: Hertz::mhz(8),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll = Some(Pll {
        src: PllSource::HSE,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL9,
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.ls = LsConfig::default_lse();
    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(make_config());

    pac::AFIO.mapr().modify(|w| {
        w.set_swj_cfg(0b0000_0010); // this is equal to __HAL_AFIO_REMAP_SWJ_NOJTAG() in C
        w.set_spi1_remap(true);
    });

    info!("Little car started!");

    let mut idle = Idle::new(p.PC13);

    let mut car = Car::new(p.TIM2, p.PA1, p.PA2, p.PB0, p.PB1, p.PB10, p.PB11);

    let bluetooth = Bluetooth::new(p.USART1, Irqs, p.PA10, p.DMA1_CH5);

    let mut servo = Servo::new(p.TIM1, p.PA11);
    servo.set_angle(90);

    let ultrasound = Ultrasound::new(p.PA4, p.PA3, p.EXTI3);

    spawner.spawn(handle_ultrasound(ultrasound)).unwrap();
    spawner.spawn(handle_bluetooth(bluetooth)).unwrap();

    loop {
        match CONTROL_CHANNEL.receive().await {
            Control::CarStop => {
                car.stop();
                idle.set_idle(true);
            }
            Control::CarForward => car.forward(),
            Control::CarBackward => car.backward(),
            Control::CarTurnLeft => car.turn_left(),
            Control::CarTurnRight => car.turn_right(),
            Control::CarForwardLeft => car.forward_left(),
            Control::CarForwardRight => car.forward_right(),
            Control::CarBackwardLeft => car.backward_left(),
            Control::CarBackwardRight => car.backward_right(),
            Control::CarSetSpeed(speed) => car.set_speed(speed),
            Control::ServoSetAngle(angle) => servo.set_angle(angle),
            Control::ControlModeSet(mode) => {
                servo.set_angle(90);
                car.stop();
                match mode {
                    ControlMode::AutoTrack => {
                        car.set_speed(50);
                    }
                    ControlMode::Bluetooth => {
                        car.set_speed(90);
                        idle.set_idle(true);
                    }
                }
                CONTROL_MODE_WATCH.sender().send(mode);
            }
            Control::IdleSetIdle(b) => idle.set_idle(b),
            Control::IdleSetActive(active) => idle.set_enable(active),
        }
    }
}

#[embassy_executor::task]
async fn handle_bluetooth(mut bluetooth: Bluetooth) {
    let mut enabled = true;
    let mut recv = CONTROL_MODE_WATCH.receiver().unwrap();

    CONTROL_CHANNEL.send(Control::IdleSetActive(true)).await;

    loop {
        let control = async {
            match bluetooth.receive().await {
                Ok(action) => {
                    debug!("receive action: {:?}", action);

                    if let BluetoothAction::ControlMode(mode) = action {
                        CONTROL_CHANNEL.send(Control::ControlModeSet(mode)).await;
                        return;
                    }

                    if !enabled {
                        return;
                    }

                    match action {
                        BluetoothAction::Stop => {
                            CONTROL_CHANNEL.send(Control::CarStop).await;
                        }
                        BluetoothAction::Forward => {
                            CONTROL_CHANNEL.send(Control::CarForward).await;
                        }
                        BluetoothAction::Backward => {
                            CONTROL_CHANNEL.send(Control::CarBackward).await;
                        }
                        BluetoothAction::TurnLeft => {
                            CONTROL_CHANNEL.send(Control::CarTurnLeft).await;
                        }
                        BluetoothAction::TurnRight => {
                            CONTROL_CHANNEL.send(Control::CarTurnRight).await;
                        }
                        BluetoothAction::ForwardLeft => {
                            CONTROL_CHANNEL.send(Control::CarForwardLeft).await;
                        }
                        BluetoothAction::ForwardRight => {
                            CONTROL_CHANNEL.send(Control::CarForwardRight).await;
                        }
                        BluetoothAction::BackwardLeft => {
                            CONTROL_CHANNEL.send(Control::CarBackwardLeft).await;
                        }
                        BluetoothAction::BackwardRight => {
                            CONTROL_CHANNEL.send(Control::CarBackwardRight).await;
                        }
                        BluetoothAction::Speed(speed) => {
                            CONTROL_CHANNEL.send(Control::CarSetSpeed(speed)).await;
                        }
                        BluetoothAction::ControlMode(mode) => {
                            CONTROL_CHANNEL.send(Control::ControlModeSet(mode)).await;
                        }
                    }

                    CONTROL_CHANNEL
                        .send(Control::IdleSetIdle(matches!(
                            action,
                            BluetoothAction::Stop
                        )))
                        .await;
                }
                Err(act) => error!("unknown action: {:?}", act),
            }
        };
        match select(recv.changed(), control).await {
            Either::First(mode) => {
                debug!("bluetooth recv mode: {}", mode);
                enabled = matches!(mode, ControlMode::Bluetooth);
            }
            Either::Second(_) => {}
        }
    }
}

#[embassy_executor::task]
async fn handle_ultrasound(mut ultrasound: Ultrasound) {
    const SERVO_SET_ANGLE_SECS: u64 = 1;
    const CAR_TURN_MILLIS: u64 = 1300;

    let mut enabled = false;
    let mut recv = CONTROL_MODE_WATCH.receiver().unwrap();

    loop {
        let control = async {
            if !enabled {
                return future::pending().await;
            }

            let distance = ultrasound.distance().await;

            if distance < 10. {
                CONTROL_CHANNEL.send(Control::CarStop).await;
                CONTROL_CHANNEL.send(Control::ServoSetAngle(0)).await;
                Timer::after_secs(SERVO_SET_ANGLE_SECS).await;

                let distance = ultrasound.distance().await;

                if distance < 10. {
                    CONTROL_CHANNEL.send(Control::ServoSetAngle(180)).await;
                    Timer::after_secs(SERVO_SET_ANGLE_SECS).await;

                    let distance = ultrasound.distance().await;

                    if distance < 10. {
                        warn!("进入了死胡同！");
                        CONTROL_CHANNEL
                            .send(Control::ControlModeSet(ControlMode::Bluetooth))
                            .await;
                        return;
                    } else {
                        CONTROL_CHANNEL.send(Control::CarTurnLeft).await;
                        CONTROL_CHANNEL.send(Control::ServoSetAngle(90)).await;
                        Timer::after_millis(CAR_TURN_MILLIS).await;
                    }
                } else {
                    CONTROL_CHANNEL.send(Control::CarTurnRight).await;
                    CONTROL_CHANNEL.send(Control::ServoSetAngle(90)).await;
                    Timer::after_millis(CAR_TURN_MILLIS).await;
                }
            }

            CONTROL_CHANNEL.send(Control::CarForward).await;
            CONTROL_CHANNEL.send(Control::IdleSetIdle(false)).await;
        };

        match select(recv.changed(), control).await {
            Either::First(mode) => {
                debug!("ultrasound recv mode: {}", mode);
                enabled = matches!(mode, ControlMode::AutoTrack);
            }
            Either::Second(_) => {}
        }
    }
}
