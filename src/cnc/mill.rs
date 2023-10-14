use super::{
    block::Block,
    milling_process::{MillingError, MillingResult},
};
use nalgebra::Vector3;

#[derive(Default, Clone, Copy, Debug)]
pub enum MillType {
    #[default]
    Ball,
    Cylinder,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct MillShape {
    pub type_: MillType,
    pub diameter: f32,
}

#[derive(Default)]
pub struct Mill {
    movement_speed: Option<f32>,
    rotation_speed: Option<f32>,
    position: Vector3<f32>,
    shape: MillShape,
}

impl Mill {
    pub const MIN_MOVEMENT_SPEED: f32 = 2.0;
    pub const MAX_MOVEMENT_SPEED: f32 = 60.0;

    pub const MIN_ROTATION_SPEED: f32 = 2.0;
    pub const MAX_ROTATION_SPEED: f32 = 15.0;

    pub fn new(shape: MillShape) -> Self {
        Self {
            shape,
            ..Default::default()
        }
    }

    pub fn set_movement_speed(&mut self, speed: f32) -> MillingResult {
        if !(Self::MIN_MOVEMENT_SPEED..=Self::MAX_MOVEMENT_SPEED).contains(&speed) {
            return Err(MillingError::MovementSpeed(speed));
        }

        self.movement_speed = Some(speed);
        Ok(())
    }

    pub fn set_rotation_speed(&mut self, speed: f32) -> MillingResult {
        if !(Mill::MIN_ROTATION_SPEED..=Self::MAX_ROTATION_SPEED).contains(&speed) {
            return Err(MillingError::RotationSpeed(speed));
        }

        self.rotation_speed = Some(speed);
        Ok(())
    }

    pub fn move_slow_to(&mut self, position: Vector3<f32>) -> MillingResult {
        self.ensure_movement_and_rotation_speeds()?;
        self.position = position;
        Ok(())
    }

    pub fn move_fast_to(&mut self, position: Vector3<f32>) -> MillingResult {
        self.ensure_movement_and_rotation_speeds()?;
        self.position = position;
        Ok(())
    }

    pub fn cut(&self, block: &mut Block) -> MillingResult {
        match self.shape.type_ {
            MillType::Ball => self.cut_ball(block),
            MillType::Cylinder => self.cut_cylinder(block),
        }
    }

    fn cut_ball(&self, block: &mut Block) -> MillingResult {
        todo!()
    }

    fn cut_cylinder(&self, block: &mut Block) -> MillingResult {
        todo!()
    }

    fn ensure_movement_and_rotation_speeds(&self) -> MillingResult {
        if self.movement_speed.is_none() {
            Err(MillingError::NoMovementSpeed)
        } else if self.rotation_speed.is_none() {
            Err(MillingError::NoRotationSpeed)
        } else {
            Ok(())
        }
    }
}
