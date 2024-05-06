use fern::colors::{Color, ColoredLevelConfig};

pub fn set_up_logging() {
    // configure colors for the whole line
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::White);

    // configure colors for the severity
    let colors_level = colors_line.info(Color::Green).debug(Color::Blue);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{time} {color_line}{level} {white}{target}] {color_line}{message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                white = format_args!("\x1B[{}m", Color::White.to_fg_str()),
                time = chrono::offset::Local::now().format("%H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        // Sets log level across entire crate to remove verbose dependency information
        .level(log::LevelFilter::Warn)
        // TODO change name
        .level_for("vulkano_teapot", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}
