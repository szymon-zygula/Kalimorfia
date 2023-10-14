use super::milling_process::MillingProcess;
use std::time::Duration;

pub struct MillingPlayer<'a> {
    milling_process: &'a mut MillingProcess,
    fast_speed: f32,
    slow_speed: f32,
    instruction_interval: Duration,
}

impl<'a> MillingPlayer<'a> {
    const DEFAULT_DELTA_TIME: Duration = Duration::from_secs(1);
    const DEFAULT_SLOW_SPEED: f32 = 1.0;
    const DEFAULT_FAST_SPEED: f32 = 3.0;

    pub fn new(milling_process: &'a mut MillingProcess) -> Self {
        Self {
            milling_process,
            fast_speed: Self::DEFAULT_FAST_SPEED,
            slow_speed: Self::DEFAULT_SLOW_SPEED,
            instruction_interval: Self::DEFAULT_DELTA_TIME,
        }
    }

    pub fn step(&mut self, delta: Duration) {
        todo!()
    }

    pub fn complete(&mut self) {
        todo!()
    }
}
