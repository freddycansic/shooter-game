use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct SimplePoint {
    position: [f32; 3],
}
implement_vertex!(SimplePoint, position);

pub const CUBE: [SimplePoint; 36] = [
    SimplePoint {
        position: [-0.5, 0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, -0.5, -0.5],
    },
    SimplePoint {
        position: [0.5, -0.5, -0.5],
    },
    SimplePoint {
        position: [0.5, -0.5, -0.5],
    },
    SimplePoint {
        position: [0.5, 0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, 0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, -0.5, 0.5],
    },
    SimplePoint {
        position: [-0.5, -0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, 0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, 0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, 0.5, 0.5],
    },
    SimplePoint {
        position: [-0.5, -0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, -0.5, -0.5],
    },
    SimplePoint {
        position: [0.5, -0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, 0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, 0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, 0.5, -0.5],
    },
    SimplePoint {
        position: [0.5, -0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, -0.5, 0.5],
    },
    SimplePoint {
        position: [-0.5, 0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, 0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, 0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, -0.5, 0.5],
    },
    SimplePoint {
        position: [-0.5, -0.5, 0.5],
    },
    SimplePoint {
        position: [-0.5, 0.5, -0.5],
    },
    SimplePoint {
        position: [0.5, 0.5, -0.5],
    },
    SimplePoint {
        position: [0.5, 0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, 0.5, 0.5],
    },
    SimplePoint {
        position: [-0.5, 0.5, 0.5],
    },
    SimplePoint {
        position: [-0.5, 0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, -0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, -0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, -0.5, -0.5],
    },
    SimplePoint {
        position: [0.5, -0.5, -0.5],
    },
    SimplePoint {
        position: [-0.5, -0.5, 0.5],
    },
    SimplePoint {
        position: [0.5, -0.5, 0.5],
    },
];
