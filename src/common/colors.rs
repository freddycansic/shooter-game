use cgmath::Vector4;
use palette::{FromColor, IntoColor, Lch, ShiftHue, Srgb};

type Color = Lch;

pub fn shift_hue(color: &Lch, time: f32) -> Color {
    let shift = time % 360.0;
    color.shift_hue(shift)
}

pub fn shift_hue_from_named(color: Srgb<u8>, time: f32) -> Color {
    shift_hue(&Lch::from_color(Srgb::<f32>::from_format(color)), time)
}

pub fn to_vector4(color: Color) -> Vector4<f32> {
    let rgb: Srgb = color.into_color();

    Vector4::new(rgb.red as f32, rgb.green as f32, rgb.blue as f32, 1.0)
}
