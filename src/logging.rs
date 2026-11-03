//! Logging backend of the `log` facade (TCK-198): stderr for dev builds and the headless runner, a small rotating
//! file in the app data dir for the release app (whose `windows_subsystem = "windows"` has no console).
//!
//! The file `logs/broodbattle.log` rotates at `MAX_FILE_BYTES` into `broodbattle.log.1` (one backup kept), so the
//! logs never take more than about twice that size. Library code only uses the `log` macros; without an installed
//! backend (unit tests) they are no-ops.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};

/// Size at which the log file rotates.
pub const MAX_FILE_BYTES: u64 = 1024 * 1024;
pub const LOG_FILE_NAME: &str = "broodbattle.log";

struct FileSink {
    path: PathBuf,
    file: File,
    size: u64,
    max_bytes: u64,
}

impl FileSink {
    fn open(path: PathBuf, max_bytes: u64) -> std::io::Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let size = file.metadata()?.len();
        Ok(Self {
            path,
            file,
            size,
            max_bytes,
        })
    }

    fn write_line(&mut self, line: &str) {
        if self.size + line.len() as u64 > self.max_bytes && self.size > 0 {
            let backup = self.path.with_extension("log.1");
            let _ = self.file.flush();
            let _ = std::fs::rename(&self.path, &backup);
            match OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.path)
            {
                Ok(file) => {
                    self.file = file;
                    self.size = 0;
                }
                Err(_) => return,
            }
        }
        if self.file.write_all(line.as_bytes()).is_ok() {
            self.size += line.len() as u64;
        }
    }
}

struct Logger {
    console: bool,
    file: Option<Mutex<FileSink>>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} {:<5} {}: {}\n",
            crate::shared::time::now_ms(),
            record.level(),
            record.target(),
            record.args()
        );
        if self.console {
            eprint!("{}", line);
        }
        if let Some(file) = &self.file {
            if let Ok(mut sink) = file.lock() {
                sink.write_line(&line);
            }
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file {
            if let Ok(mut sink) = file.lock() {
                let _ = sink.file.flush();
            }
        }
    }
}

fn install(logger: Logger) {
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(LevelFilter::Info);
    }
}

/// Console (stderr) logging: dev tools and the headless runner.
pub fn init_console() {
    install(Logger {
        console: true,
        file: None,
    });
}

/// Logging of the app: release builds write `<log_dir>/broodbattle.log` (rotating); dev builds log to the console.
/// If the file cannot be opened the app logs to stderr only.
pub fn init_app(log_dir: &Path) {
    if cfg!(debug_assertions) {
        init_console();
        return;
    }
    match FileSink::open(log_dir.join(LOG_FILE_NAME), MAX_FILE_BYTES) {
        Ok(sink) => install(Logger {
            console: false,
            file: Some(Mutex::new(sink)),
        }),
        Err(e) => {
            init_console();
            log::error!("cannot open log file in '{}': {}", log_dir.display(), e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_sink_rotates_and_keeps_one_backup() {
        let dir = std::env::temp_dir().join(format!("broodbattle-log-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join(LOG_FILE_NAME);
        let mut sink = FileSink::open(path.clone(), 100).unwrap();
        for i in 0..10 {
            sink.write_line(&format!("line {:02} ..................\n", i));
        }
        let _ = sink.file.flush();
        let current = std::fs::read_to_string(&path).unwrap();
        let backup = std::fs::read_to_string(path.with_extension("log.1")).unwrap();
        assert!(
            current.len() <= 100 && backup.len() <= 100,
            "files stay below the rotation size"
        );
        assert!(current.contains("line 09"), "the newest line is in the current file");
        assert!(
            !current.contains("line 00") && !backup.contains("line 00"),
            "only one backup is kept"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
