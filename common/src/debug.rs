use fern::colors::ColoredLevelConfig;
use log::LevelFilter;
use nalgebra::Vector3;

use crate::colors::Color;

#[derive(Clone, Debug)]
pub struct DebugCuboid {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
    pub color: Color,
}

pub fn set_up_logging() {
    // configure colors for the whole line
    let colors_line = ColoredLevelConfig::new()
        .error(fern::colors::Color::Red)
        .warn(fern::colors::Color::Yellow)
        .info(fern::colors::Color::White)
        .debug(fern::colors::Color::White)
        .trace(fern::colors::Color::White);

    // configure colors for the severity
    let colors_level = colors_line
        .info(fern::colors::Color::Green)
        .debug(fern::colors::Color::Blue);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{time} {color_line}{level} {white}{target}] {color_line}{message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                white = format_args!("\x1B[{}m", fern::colors::Color::White.to_fg_str()),
                time = chrono::offset::Local::now().format("%H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        // Sets log level across entire crate to remove verbose dependency information
        .level(log::LevelFilter::Trace)
        .level_for("egui_winit", LevelFilter::Off)
        .level_for("egui", LevelFilter::Off)
        .level_for("egui_glium", LevelFilter::Off)
        .level_for("calloop", LevelFilter::Off)
        .level_for("arboard", LevelFilter::Off)
        .level_for("tracing", LevelFilter::Off)
        .level_for("winit", LevelFilter::Off)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}
