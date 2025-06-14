use std::sync;
use std::io;

use log;

struct Logger<T> {
    level: log::LevelFilter,
    output: sync::Mutex<T>
}

impl<T: Send + io::Write> Logger<T> {
    pub fn new(output: T, level: log::LevelFilter) -> Logger<io::LineWriter<T>> {
        Logger {
            level: level,
            output: sync::Mutex::new(io::LineWriter::new(output))
        }
    }
}

impl<T: Send + io::Write> log::Log for Logger<T> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level &&
            metadata.target().starts_with("rkllm_shell")
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let Ok(ref mut writer) = self.output.lock() {
                let _ = writeln!(writer, "{}", record.args());
            }
        }
    }
    
    fn flush(&self) {}
}

pub fn init(verbosity: &u8) -> Result<(), log::SetLoggerError> {
        let max_log_level = match verbosity{
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace
        };
    let _ = log::set_boxed_logger(Box::new( Logger::new(io::stderr(),
        max_log_level
    )));
    log::set_max_level(max_log_level);
    Ok(())
}
