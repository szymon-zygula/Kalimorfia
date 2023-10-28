use super::{block::Block, location::Location, mill::Mill, program::Program};
use nalgebra::Vector3;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum MillInstruction {
    RotationSpeed(f32),
    MovementSpeed(f32),
    MoveFast(Location),
    MoveSlow(Location),
}

impl MillInstruction {
    pub fn to_str(&self) -> String {
        match self {
            MillInstruction::RotationSpeed(_) => unimplemented!(),
            MillInstruction::MovementSpeed(_) => unimplemented!(),
            MillInstruction::MoveFast(_) => unimplemented!(),
            MillInstruction::MoveSlow(location) => {
                format!("G01{}", location.to_str())
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum MillingError {
    #[error("moving a mill which has no movement speed")]
    NoMovementSpeed,
    #[error("moving a mill without rotation speed")]
    NoRotationSpeed,
    #[error("lower non-cutting part of the mill is being pushed into the material")]
    LowerDeadZoneCollision,
    #[error("upper non-cutting part of the mill is being pushed into the material")]
    UpperDeadZoneCollision,
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
    current_instruction: usize,
}

impl MillingProcess {
    pub fn new(mill: Mill, program: Program, block: Block) -> Self {
        Self {
            mill,
            program,
            current_instruction: 0,
            block,
        }
    }

    pub fn execute_next_instruction(&mut self) -> MillingResult {
        if self.done() {
            return Ok(());
        }

        let instruction = self.current_instruction().clone();
        self.current_instruction += 1;

        match instruction {
            MillInstruction::RotationSpeed(speed) => self.mill.set_rotation_speed(speed),
            MillInstruction::MovementSpeed(speed) => self.mill.set_movement_speed(speed),
            MillInstruction::MoveFast(location) => {
                self.move_fast_to(&location.relative_to(self.mill.position()))
            }
            MillInstruction::MoveSlow(location) => {
                self.move_slow_to(&location.relative_to(self.mill.position()))
            }
        }
    }

    fn move_fast_to(&mut self, _location: &Vector3<f32>) -> MillingResult {
        unimplemented!("Fast moves are not supported")
    }

    fn move_slow_to(&mut self, location: &Vector3<f32>) -> MillingResult {
        let Some(direction) = (location - self.mill.position()).try_normalize(0.0) else {
            self.mill.cut(&mut self.block, &Vector3::zeros())?;
            return Ok(());
        };
        let min_sample = self.block.sample_size().min();
        let distance = Vector3::metric_distance(location, self.mill.position());
        let step_count = std::cmp::max((distance / min_sample).ceil() as usize, 1);
        let step = distance / step_count as f32;
        let initial_position = *self.mill.position();

        for step_idx in 0..=step_count {
            let position = initial_position + direction * step_idx as f32 * step;
            self.mill.move_to(position)?;
            self.mill.cut(&mut self.block, &direction)?;
        }

        // Make up for numerical errors
        self.mill.move_to(*location)?;
        self.mill.cut(&mut self.block, &direction)?;

        Ok(())
    }

    fn current_instruction(&self) -> &MillInstruction {
        &self.program.instructions()[self.current_instruction]
    }

    pub fn current_instruction_idx(&self) -> usize {
        self.current_instruction
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    fn current_instruction_length(&self) -> f32 {
        if let MillInstruction::MoveSlow(location) =
            &self.program.instructions()[self.current_instruction]
        {
            location.f32_dist(self.mill.position())
        } else {
            0.0
        }
    }

    pub fn execute_next_instruction_partially(&mut self, mut dist_left: f32) -> MillingResult {
        if self.done() {
            return Ok(());
        }

        while dist_left > 0.0 && !self.done() {
            let instruction = self.current_instruction().clone();
            let current_instruction_length = self.current_instruction_length();

            if dist_left >= current_instruction_length {
                self.current_instruction += 1;
            }
            let current_dist = f32::min(current_instruction_length, dist_left);
            dist_left -= current_dist;

            if let MillInstruction::MoveSlow(location) = instruction {
                let target = location.move_toward(self.mill.position(), current_dist);
                self.move_slow_to(&target)?;
            } else {
                self.execute_next_instruction()?;
            }
        }

        Ok(())
    }

    pub fn done(&self) -> bool {
        self.current_instruction == self.program.instructions().len()
    }

    pub fn retake_all(self) -> (Mill, Program, Block) {
        (self.mill, self.program, self.block)
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn block_mut(&mut self) -> &mut Block {
        &mut self.block
    }

    pub fn mill(&self) -> &Mill {
        &self.mill
    }

    pub fn mill_mut(&mut self) -> &mut Mill {
        &mut self.mill
    }
}
