use input::Input;

pub trait Camera {
    fn update(input: &Input, deltatime: f32);  
};
