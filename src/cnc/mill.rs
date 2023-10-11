use crate::cnc::location::Location;
use nalgebra::Vector3;
use thiserror::Error;

#[derive(Error, Debug)]
enum MillingError {
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

pub enum MillInstruction {
    RotationSpeed(f32),
    MovementSpeed(f32),
    MoveFast(Location),
    MoveSlow(Location),
}

#[derive(Default)]
struct Mill {
    movement_speed: Option<f32>,
    rotation_speed: Option<f32>,
    position: Vector3<f32>,
}

impl Mill {
    const MIN_MOVEMENT_SPEED: f32 = 2.0;
    const MAX_MOVEMENT_SPEED: f32 = 60.0;

    const MIN_ROTATION_SPEED: f32 = 2.0;
    const MAX_ROTATION_SPEED: f32 = 15.0;

    fn new() -> Self {
        Self::default()
    }

    fn execute_instruction(&mut self, instruction: &MillInstruction) -> Result<(), MillingError> {
        match instruction {
            MillInstruction::RotationSpeed(speed) => {
                if Self::MIN_ROTATION_SPEED > *speed || *speed > Self::MAX_ROTATION_SPEED {
                    return Err(MillingError::RotationSpeed(*speed));
                }

                self.rotation_speed = Some(*speed);
                Ok(())
            }
            MillInstruction::MovementSpeed(speed) => {
                if Self::MIN_MOVEMENT_SPEED > *speed || *speed > Self::MAX_MOVEMENT_SPEED {
                    return Err(MillingError::MovementSpeed(*speed));
                }

                self.movement_speed = Some(*speed);
                Ok(())
            }
            MillInstruction::MoveFast(location) => {
                self.ensure_movement_and_rotation_speeds()?;
                todo!()
            }
            MillInstruction::MoveSlow(location) => {
                self.ensure_movement_and_rotation_speeds()?;
                todo!()
            }
        }
    }

    fn ensure_movement_and_rotation_speeds(&self) -> Result<(), MillingError> {
        if self.movement_speed.is_none() {
            Err(MillingError::NoMovementSpeed)
        } else if self.rotation_speed.is_none() {
            Err(MillingError::NoRotationSpeed)
        } else {
            Ok(())
        }
    }
}
