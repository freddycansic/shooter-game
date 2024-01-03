use crate::model::Model;

pub struct Scene {
    pub models: Vec<Model>,
}

impl Scene {
    fn render(&self) {
        // build per-frame uniforms

        // draw each primitive
        for model in &self.models {
            //
            model.render();
        }
    }
}
