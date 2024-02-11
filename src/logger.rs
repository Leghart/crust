use chrono::Utc;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use text_colorizer::Colorize;

use crate::LOGGER;

/// Main, custom logger in application.
pub struct Logger;

/// Set log level of Logger with requested enum-value.
pub fn init(level: &LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(*level))
}

/// Set of methods to make real logger from custom Logger struct.
impl Log for Logger {
    /// TODO? (discuss) In `clap-verbosity-flag` crate logger is always enabled (even without
    /// TODO? '-v' flag). To disable logger, user must pass '-q/--quiet' flag.
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    /// Specifies how each level is displayed.
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{}", record.args());
            match record.level() {
                Level::Info => println!("{}", message),
                Level::Warn => println!("{}", message.yellow()),
                Level::Error => println!("{}", message.red()),
                Level::Debug => println!(
                    "[{}] {}",
                    Utc::now().format("%Y-%m-%d %H:%M:%S"),
                    message.magenta()
                ),
                Level::Trace => println!(
                    "[{}] {}",
                    Utc::now().format("%Y-%m-%d %H:%M:%S"),
                    message.blue()
                ),
            }
        }
    }

    fn flush(&self) {}
}
