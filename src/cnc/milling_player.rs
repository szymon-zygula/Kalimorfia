use super::milling_process::{MillingProcess, MillingResult};
use std::time::Instant;

pub struct MillingPlayer {
    milling_process: MillingProcess,
    pub slow_speed: f32,
    last_step: Instant,
}

impl MillingPlayer {
    const DEFAULT_SLOW_SPEED: f32 = 10.0;

    pub fn new(milling_process: MillingProcess) -> Self {
        Self {
            milling_process,
            slow_speed: Self::DEFAULT_SLOW_SPEED,
            last_step: Instant::now(),
        }
    }

    pub fn full_step(&mut self) -> MillingResult {
        self.milling_process.execute_next_instruction()
    }

    pub fn reset_timer(&mut self) {
        self.last_step = Instant::now();
    }

    pub fn step(&mut self) -> MillingResult {
        let now = Instant::now();
        let delta = (now - self.last_step).as_secs_f32();
        self.last_step = now;
        self.milling_process
            .execute_next_instruction_partially(delta * self.slow_speed)?;

        Ok(())
    }

    pub fn complete(&mut self) -> MillingResult {
        while !self.milling_process.done() {
            self.milling_process.execute_next_instruction()?;
        }

        Ok(())
    }

    pub fn milling_process(&self) -> &MillingProcess {
        &self.milling_process
    }

    pub fn milling_process_mut(&mut self) -> &mut MillingProcess {
        &mut self.milling_process
    }

    pub fn take(self) -> MillingProcess {
        self.milling_process
    }
}
