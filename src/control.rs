use defmt::Format;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, watch::Watch};

pub type ControlChannel = Channel<CriticalSectionRawMutex, Control, 10>;

pub type ControlModeWatch = Watch<CriticalSectionRawMutex, ControlMode, 10>;

pub enum Control {
    Car(CarControl),
    ServoSetAngle(u8),
    IdleSetIdle(bool),
    IdleSetActive(bool),
    ControlModeSet(ControlMode),
}

#[derive(Format)]
pub enum CarControl {
    Stop,
    Forward,
    Backward,
    TurnLeft,
    TurnRight,
    ForwardLeft,
    ForwardRight,
    BackwardLeft,
    BackwardRight,
    SetSpeed(u8),
}

#[derive(Clone, Copy, Format)]
pub enum ControlMode {
    AutoTrack,
    Bluetooth,
}
