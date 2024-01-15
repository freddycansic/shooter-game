use cgmath::Vector4;

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "assets/shaders/default.vert",
        linalg_type: "cgmath"
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "assets/shaders/default.frag",
        linalg_type: "cgmath"
    }
}

impl Default for fs::Light {
    fn default() -> Self {
        Self {
            position: Vector4::new(0.0, 0.0, 0.0, 1.0),
            color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            intensity: 1.0,
        }
    }
}
