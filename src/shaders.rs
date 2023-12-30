pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "assets/shaders/default.vert",
        // TODO linalg_type: "cgmath"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "assets/shaders/default.frag",
    }
}

impl Default for fs::Light {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0].into(),
            color: [1.0, 1.0, 1.0].into(),
            intensity: 1.0,
        }
    }
}
