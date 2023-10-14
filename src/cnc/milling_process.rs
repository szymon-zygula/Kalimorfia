use super::{block::Block, location::Location, mill::Mill, program::Program};
use thiserror::Error;

pub enum MillInstruction {
    RotationSpeed(f32),
    MovementSpeed(f32),
    MoveFast(Location),
    MoveSlow(Location),
}

#[derive(Error, Debug)]
pub enum MillingError {
    #[error("moving a mill which has no movement speed")]
    NoMovementSpeed,
    #[error("moving a mill without rotation speed")]
    NoRotationSpeed,
    #[error("non-cutting part of the mill is being pushed into the material")]
    DeadZoneCollision,
    #[error("the mill is lowered too deeply")]
    CutTooDeep,
    #[error("movement speed {0} not in allowed range")]
    MovementSpeed(f32),
    #[error("rotation speed {0} not in allowed range")]
    RotationSpeed(f32),
}

pub type MillingResult = Result<(), MillingError>;

pub struct MillingProcess {
    mill: Mill,
    program: Program,
    block: Block,
}

impl MillingProcess {
    pub fn new(mill: Mill, program: Program, block: Block) -> Self {
        Self {
            mill,
            program,
            block,
        }
    }

    pub fn execute_next_instruction(&mut self, instruction: &MillInstruction) -> MillingResult {
        match instruction {
            MillInstruction::RotationSpeed(speed) => self.mill.set_rotation_speed(*speed),
            MillInstruction::MovementSpeed(speed) => self.mill.set_movement_speed(*speed),
            MillInstruction::MoveFast(location) => self.move_fast_to(location),
            MillInstruction::MoveSlow(location) => self.move_slow_to(location),
        }
    }

    fn move_fast_to(&mut self, location: &Location) -> MillingResult {
        todo!("Fast moves are not supported")
    }

    fn move_slow_to(&mut self, location: &Location) -> MillingResult {
        todo!()
    }

    pub fn execute_next_instruction_partially(&mut self) {
        todo!()
    }

    pub fn retake_all(self) -> (Mill, Program, Block) {
        (self.mill, self.program, self.block)
    }
}
