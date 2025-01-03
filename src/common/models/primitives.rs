use glium::implement_vertex;
#[derive(Copy, Clone)]
pub struct SimplePoint {
    position: [f32; 3],
}
implement_vertex!(SimplePoint, position);

pub const CUBE: [SimplePoint; 36] = [
    SimplePoint {
        position: [-1.0, 1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, -1.0, -1.0],
    },
    SimplePoint {
        position: [1.0, -1.0, -1.0],
    },
    SimplePoint {
        position: [1.0, -1.0, -1.0],
    },
    SimplePoint {
        position: [1.0, 1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, 1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, -1.0, 1.0],
    },
    SimplePoint {
        position: [-1.0, -1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, 1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, 1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, 1.0, 1.0],
    },
    SimplePoint {
        position: [-1.0, -1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, -1.0, -1.0],
    },
    SimplePoint {
        position: [1.0, -1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, 1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, 1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, 1.0, -1.0],
    },
    SimplePoint {
        position: [1.0, -1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, -1.0, 1.0],
    },
    SimplePoint {
        position: [-1.0, 1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, 1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, 1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, -1.0, 1.0],
    },
    SimplePoint {
        position: [-1.0, -1.0, 1.0],
    },
    SimplePoint {
        position: [-1.0, 1.0, -1.0],
    },
    SimplePoint {
        position: [1.0, 1.0, -1.0],
    },
    SimplePoint {
        position: [1.0, 1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, 1.0, 1.0],
    },
    SimplePoint {
        position: [-1.0, 1.0, 1.0],
    },
    SimplePoint {
        position: [-1.0, 1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, -1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, -1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, -1.0, -1.0],
    },
    SimplePoint {
        position: [1.0, -1.0, -1.0],
    },
    SimplePoint {
        position: [-1.0, -1.0, 1.0],
    },
    SimplePoint {
        position: [1.0, -1.0, 1.0],
    },
];
