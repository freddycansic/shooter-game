use crate::camera::Camera;
use crate::model::ModelInstance;

pub struct Scene {
    pub models: Vec<ModelInstance>,
    pub camera: Camera,
}

impl Scene {
    pub fn new(camera: Camera) -> Self {
        Self {
            models: vec![],
            camera,
        }
    }

    // fn render(&self) {
    //     // build per-frame uniforms
    //
    //     // draw each primitive
    //     for model in &self.models {
    //         //
    //         model.render();
    //     }
    // }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(Camera::default())
    }
}
