use color_eyre::Result;
use fern::colors::{Color, ColoredLevelConfig};
use log::{debug, error, info, trace, warn};
use std::sync::Arc;
use vulkano::instance::debug::{
    DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
    DebugUtilsMessengerCallback, DebugUtilsMessengerCallbackData, DebugUtilsMessengerCreateInfo,
};
use vulkano::instance::Instance;

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
                "{color_line}[{time} {level} {target}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
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

    debug!("set up logging");
}

pub fn create_debug_callback(
    instance: Arc<Instance>,
    min_severity: DebugUtilsMessageSeverity,
    enabled_types: DebugUtilsMessageType,
) -> Result<DebugUtilsMessenger> {
    Ok(unsafe {
        DebugUtilsMessenger::new(
            instance,
            DebugUtilsMessengerCreateInfo {
                message_severity: enabled_severity_from_min(min_severity),
                message_type: enabled_types,
                ..DebugUtilsMessengerCreateInfo::user_callback(DebugUtilsMessengerCallback::new(
                    callback,
                ))
            },
        )?
    })
}

fn callback(
    message_severity: DebugUtilsMessageSeverity,
    message_type: DebugUtilsMessageType,
    callback_data: DebugUtilsMessengerCallbackData<'_>,
) {
    let ty = match message_type {
        ty if ty.intersects(DebugUtilsMessageType::GENERAL) => "general",
        ty if ty.intersects(DebugUtilsMessageType::PERFORMANCE) => "performance",
        ty if ty.intersects(DebugUtilsMessageType::VALIDATION) => "validation",
        _ => "unknown",
    };

    match message_severity {
        severity if severity.intersects(DebugUtilsMessageSeverity::ERROR) => error!(
            "[{}] [{}] {}",
            callback_data.message_id_name.unwrap_or("unknown"),
            ty,
            callback_data.message
        ),
        severity if severity.intersects(DebugUtilsMessageSeverity::WARNING) => warn!(
            "[{}] [{}] {}",
            callback_data.message_id_name.unwrap_or("unknown"),
            ty,
            callback_data.message
        ),
        severity if severity.intersects(DebugUtilsMessageSeverity::INFO) => info!(
            "[{}] [{}] {}",
            callback_data.message_id_name.unwrap_or("unknown"),
            ty,
            callback_data.message
        ),
        _ => trace!(
            "[{}] [{}] {}",
            callback_data.message_id_name.unwrap_or("unknown"),
            ty,
            callback_data.message
        ),
    };
}

fn enabled_severity_from_min(min_severity: DebugUtilsMessageSeverity) -> DebugUtilsMessageSeverity {
    match min_severity {
        DebugUtilsMessageSeverity::ERROR => DebugUtilsMessageSeverity::ERROR,
        DebugUtilsMessageSeverity::WARNING => {
            DebugUtilsMessageSeverity::ERROR | DebugUtilsMessageSeverity::WARNING
        }
        DebugUtilsMessageSeverity::INFO => {
            DebugUtilsMessageSeverity::ERROR
                | DebugUtilsMessageSeverity::WARNING
                | DebugUtilsMessageSeverity::INFO
        }
        DebugUtilsMessageSeverity::VERBOSE => {
            DebugUtilsMessageSeverity::ERROR
                | DebugUtilsMessageSeverity::WARNING
                | DebugUtilsMessageSeverity::INFO
                | DebugUtilsMessageSeverity::VERBOSE
        }
        _ => panic!("unknown DebugUtilsMessageSeverity defined?"),
    }
}
