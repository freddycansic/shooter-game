use cgmath::{InnerSpace, Point3, Vector3};

pub struct Player {
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub position: Point3<f32>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            direction: Vector3::new(1.0, 0.0, 0.0),
            position: Point3::new(0.0, 0.0, 0.0),
        }
    }

    const MAX_VELOCITY: f32 = 10.0;

    pub fn update(&mut self, deltatime: f32) {
        let speed = 10.0 * deltatime;

        self.velocity += self.acceleration * speed;

        let velocity_magnitute = self.velocity.magnitude2();
        if velocity_magnitute > Self::MAX_VELOCITY {
            self.velocity = self.velocity * (Self::MAX_VELOCITY / velocity_magnitute);
        }

        self.position += self.velocity;
    }
}
