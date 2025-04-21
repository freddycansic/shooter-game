use image::Rgb;
use palette::{FromColor, IntoColor, LabHue, Lch, ShiftHue, Srgb, rgb::Rgba};
use rapier3d::na::{Vector3, Vector4};

pub type Color = Lch;

pub trait ColorExt {
    fn shift_hue_by_time(&self, time: f32) -> Self;
    fn from_named(color: Srgb<u8>) -> Self;
    fn to_rgb_vector4(self) -> Vector4<f32>;
    fn to_rgb_vector3(self) -> Vector3<f32>;
    fn to_rgb_components_tuple(self) -> (f32, f32, f32, f32);
}

pub const SELECTED: Color = Color::new_const(
    73.90410404549763,
    75.77322477206486,
    LabHue::new(69.0500156411789),
);

impl ColorExt for Color {
    fn shift_hue_by_time(&self, time: f32) -> Color {
        let shift = time % 360.0;
        self.shift_hue(shift)
    }

    fn from_named(named: Srgb<u8>) -> Color {
        Lch::from_color(Srgb::<f32>::from_format(named))
    }

    fn to_rgb_vector4(self) -> Vector4<f32> {
        let vec3 = self.to_rgb_vector3();

        Vector4::new(vec3.x, vec3.y, vec3.z, 1.0)
    }

    fn to_rgb_vector3(self) -> Vector3<f32> {
        let rgb: Srgb = self.into_color();

        Vector3::new(rgb.red, rgb.green, rgb.blue)
    }

    fn to_rgb_components_tuple(self) -> (f32, f32, f32, f32) {
        let vector_components = self.to_rgb_vector4();

        (
            vector_components[0],
            vector_components[1],
            vector_components[2],
            vector_components[3],
        )
    }
}
