use cgmath::Vector4;
use palette::{FromColor, IntoColor, Lch, ShiftHue, Srgb};

pub type Color = Lch;

pub fn shift_hue(color: &Lch, time: f32) -> Color {
    let shift = time % 360.0;
    color.shift_hue(shift)
}

pub fn shift_hue_from_named(color: Srgb<u8>, time: f32) -> Color {
    shift_hue(&from_named(color), time)
}

pub trait ColorExt {
    fn to_rgb_vector4(self) -> Vector4<f32>;
}

impl ColorExt for Color {
    fn to_rgb_vector4(self) -> Vector4<f32> {
        let rgb: Srgb = self.into_color();

        Vector4::new(rgb.red, rgb.green, rgb.blue, 1.0)
    }
}

pub fn from_named(named: Srgb<u8>) -> Color {
    Lch::from_color(Srgb::<f32>::from_format(named))
}
