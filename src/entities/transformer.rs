use super::{
    basic::{Orientation, Scale, Translation},
    entity::SceneObject,
};
use nalgebra::{Matrix4, Point3};

pub struct Transformer {
    transformee: Box<dyn SceneObject>,
    rotation: Orientation,
    translation: Translation,
    scale: Scale,
}

impl Transformer {
    pub fn new(transformee: Box<dyn SceneObject>) -> Transformer {
        Transformer {
            transformee,
            rotation: Orientation::new(),
            translation: Translation::new(),
            scale: Scale::new(),
        }
    }
}

impl SceneObject for Transformer {
    fn location(&self) -> Point3<f32> {
        self.transformee.location() + self.translation.translation
    }

    fn draw(&self, projection_transform: &Matrix4<f32>, view_transform: &Matrix4<f32>) {
        let transform =
            self.translation.as_matrix() * self.rotation.as_matrix() * self.scale.as_matrix();

        self.transformee
            .draw(projection_transform, &(view_transform * transform));
    }
}
