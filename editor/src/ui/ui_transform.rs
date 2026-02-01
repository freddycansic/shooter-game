use crate::ui::Show;
use common::maths;
use common::maths::Transform;
use egui_glium::egui_winit::egui;
use egui_glium::egui_winit::egui::Ui;
use nalgebra::UnitQuaternion;

const MAX_DECIMALS: usize = 2;

impl Show for Transform {
    fn show(&mut self, ui: &mut Ui) {
        show_translation(self, ui);
        ui.separator();
        show_rotation(self, ui);
        ui.separator();
        show_scale(self, ui);
    }
}

fn show_translation(transform: &mut Transform, ui: &mut Ui) {
    ui.label("Translation");

    let mut translation = transform.translation();
    let mut changed = false;

    let speed = 0.1;

    changed |= ui
        .add(
            egui::DragValue::new(&mut translation.x)
                .speed(speed)
                .max_decimals(MAX_DECIMALS),
        )
        .changed();
    changed |= ui
        .add(
            egui::DragValue::new(&mut translation.y)
                .speed(speed)
                .max_decimals(MAX_DECIMALS),
        )
        .changed();
    changed |= ui
        .add(
            egui::DragValue::new(&mut translation.z)
                .speed(speed)
                .max_decimals(MAX_DECIMALS),
        )
        .changed();

    if changed {
        transform.set_translation(translation);
    }
}

fn show_rotation(transform: &mut Transform, ui: &mut Ui) {
    ui.label("Rotation");

    let (mut rotation_x, mut rotation_y, mut rotation_z) = transform.rotation().euler_angles();
    rotation_x = maths::to_degrees(rotation_x);
    rotation_y = maths::to_degrees(rotation_y);
    rotation_z = maths::to_degrees(rotation_z);

    let mut changed = false;

    let speed = 0.5;
    let range = 0..=360;

    changed |= ui
        .add(
            egui::DragValue::new(&mut rotation_x)
                .speed(speed)
                .range(range.clone())
                .max_decimals(MAX_DECIMALS),
        )
        .changed();
    changed |= ui
        .add(
            egui::DragValue::new(&mut rotation_y)
                .speed(speed)
                .range(range.clone())
                .max_decimals(MAX_DECIMALS),
        )
        .changed();
    changed |= ui
        .add(
            egui::DragValue::new(&mut rotation_z)
                .speed(speed)
                .range(range)
                .max_decimals(MAX_DECIMALS),
        )
        .changed();

    if changed {
        rotation_x = maths::to_radians(rotation_x);
        rotation_y = maths::to_radians(rotation_y);
        rotation_z = maths::to_radians(rotation_z);

        transform.set_rotation(UnitQuaternion::from_euler_angles(rotation_x, rotation_y, rotation_z));
    }
}

fn show_scale(transform: &mut Transform, ui: &mut Ui) {
    ui.label("Scale");

    let mut scale = transform.scale();

    let changed = ui
        .add(egui::DragValue::new(&mut scale).speed(0.01).max_decimals(MAX_DECIMALS))
        .changed();

    if changed && scale != 0.0 {
        transform.set_scale(scale);
    }
}
