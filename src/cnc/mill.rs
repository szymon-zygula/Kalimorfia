use super::{
    block::Block,
    milling_process::{MillingError, MillingResult},
};
use nalgebra::{vector, Vector3};

#[derive(Default, Clone, Copy, Debug)]
pub enum CutterShape {
    #[default]
    Ball,
    Cylinder,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Cutter {
    pub height: f32,
    pub shape: CutterShape,
    pub diameter: f32,
}

#[derive(Default)]
pub struct Mill {
    movement_speed: Option<f32>,
    rotation_speed: Option<f32>,
    position: Vector3<f32>,
    pub cutter: Cutter,
}

impl Mill {
    pub const BALL_DOWN_ALLOWED_DOT: f32 = 0.99;

    pub const MIN_MOVEMENT_SPEED: f32 = 2.0;
    pub const MAX_MOVEMENT_SPEED: f32 = 60.0;

    pub const MIN_ROTATION_SPEED: f32 = 2.0;
    pub const MAX_ROTATION_SPEED: f32 = 15.0;

    pub fn new(shape: Cutter) -> Self {
        Self {
            cutter: shape,
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

    pub fn move_to(&mut self, position: Vector3<f32>) -> MillingResult {
        // self.ensure_movement_and_rotation_speeds()?;
        self.position = position;
        Ok(())
    }

    //
    //   ||||
    //  ||||||
    // --------> x
    //  ||||||
    //   ||||
    //
    fn milling_points(&self, block: &Block) -> Vec<(usize, usize, f32, f32)> {
        let x_diameter = self.cutter.diameter;
        let x_radius = 0.5 * x_diameter;
        let x_diameter_samples = (x_diameter / block.sample_size().x).ceil() as i32;
        let x_step = x_diameter / x_diameter_samples as f32;
        let y_diameter_samples_max = (x_diameter / block.sample_size().y).ceil() as i32;

        let mut points = Vec::with_capacity((x_diameter_samples * y_diameter_samples_max) as usize);

        for x_offset_multiple in 0..=x_diameter_samples {
            let relative_x = -x_radius + x_offset_multiple as f32 * x_step;
            let x = self.position.x + relative_x;

            let y_radius = (x_radius * x_radius - relative_x * relative_x).sqrt();
            if y_radius == 0.0 {
                continue;
            }

            let y_diameter = 2.0 * y_radius;
            let y_diameter_samples = (y_diameter / block.sample_size().y).ceil() as i32;
            let y_step = y_diameter / y_diameter_samples as f32;

            for y_offset_multiple in 0..=y_diameter_samples {
                let y = self.position.y + -y_radius + y_offset_multiple as f32 * y_step;

                let block_vec = block.mill_to_block(&vector![x, y]);
                let x_r = block_vec.x;
                let y_r = block_vec.y;

                if x_r < 0
                    || y_r < 0
                    || x_r >= block.sampling().x as i32
                    || y_r >= block.sampling().y as i32
                {
                    continue;
                }

                points.push((x_r as usize, y_r as usize, x, y))
            }
        }

        points
    }

    pub fn cut(&self, block: &mut Block, direction: &Vector3<f32>) -> MillingResult {
        match self.cutter.shape {
            CutterShape::Ball => self.cut_ball(block, direction),
            CutterShape::Cylinder => self.cut_cylinder(block, direction),
        }
    }

    fn cut_ball(&self, block: &mut Block, direction: &Vector3<f32>) -> MillingResult {
        let block_position = block.mill_to_block(&self.position.xy());

        if block.contains(&block_position)
            && block.height(block_position.x as usize, block_position.y as usize) > self.position.z
            && Vector3::dot(direction, &vector![0.0, 0.0, -1.0]) > Self::BALL_DOWN_ALLOWED_DOT
        {
            return Err(MillingError::LowerDeadZoneCollision);
        }

        let radius = 0.5 * self.cutter.diameter;
        let radius_sq = radius * radius;

        for (x_r, y_r, x, y) in self.milling_points(block) {
            if block.height(x_r, y_r) > self.cutter.height + self.position.z {
                return Err(MillingError::UpperDeadZoneCollision);
            }

            let depth = radius + self.position.z
                - (radius_sq
                    - (x - self.position.x) * (x - self.position.x)
                    - (y - self.position.y) * (y - self.position.y))
                    .sqrt();

            if depth < block.base_height {
                return Err(MillingError::CutTooDeep);
            }

            if block.height(x_r, y_r) > depth {
                *block.height_mut(x_r, y_r) = depth;
            }
        }

        Ok(())
    }

    fn cut_cylinder(&self, block: &mut Block, direction: &Vector3<f32>) -> MillingResult {
        for (x, y, _, _) in self.milling_points(block) {
            if block.height(x, y) > self.cutter.height + self.position.z {
                return Err(MillingError::UpperDeadZoneCollision);
            }

            if block.height(x, y) > self.position.z {
                if direction.z < 0.0 {
                    return Err(MillingError::LowerDeadZoneCollision);
                }

                if self.position.z < block.base_height {
                    return Err(MillingError::CutTooDeep);
                }

                *block.height_mut(x, y) = self.position.z;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn ensure_movement_and_rotation_speeds(&self) -> MillingResult {
        if self.movement_speed.is_none() {
            Err(MillingError::NoMovementSpeed)
        } else if self.rotation_speed.is_none() {
            Err(MillingError::NoRotationSpeed)
        } else {
            Ok(())
        }
    }

    pub fn position(&self) -> &Vector3<f32> {
        &self.position
    }
}
