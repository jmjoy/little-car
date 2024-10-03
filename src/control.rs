use defmt::Format;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, watch::Watch};

pub type ControlChannel = Channel<CriticalSectionRawMutex, Control, 10>;

pub type ControlModeWatch = Watch<CriticalSectionRawMutex, ControlMode, 10>;

pub enum Control {
    CarStop,
    CarForward,
    CarBackward,
    CarTurnLeft,
    CarTurnRight,
    CarForwardLeft,
    CarForwardRight,
    CarBackwardLeft,
    CarBackwardRight,
    CarSetSpeed(u8),
    ServoSetAngle(u8),
    IdleSetIdle(bool),
    IdleSetActive(bool),
    ControlModeSet(ControlMode),
}

#[derive(Clone, Copy, Format)]
pub enum ControlMode {
    AutoTrack,
    Bluetooth,
}
