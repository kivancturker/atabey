use std::path::Path;

use tracing::Dispatch;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;

use crate::error::LogError;

/// How app-log lines are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Human-readable text. The default.
    #[default]
    Text,
    /// One JSON object per line. Used in CI.
    Json,
}

/// A configured app log: the subscriber to install and the guard that flushes it.
#[derive(Debug)]
pub struct AppLog {
    /// The subscriber. Install it globally, or use it with `tracing::dispatcher::with_default`.
    pub dispatch: Dispatch,
    /// Hold this until exit: dropping it flushes buffered lines.
    pub guard: WorkerGuard,
}

/// Build (but don't install) a subscriber writing daily rolling files named
/// `atabey.YYYY-MM-DD.log` (UTC dates) in `dir`. The folder is created if missing.
///
/// Old files are left alone; deleting them is retention's job (T-07, R-24).
///
/// # Errors
/// Returns [`LogError::Setup`] if the folder or file can't be created.
pub fn app_log(dir: &Path, format: LogFormat, filter: EnvFilter) -> Result<AppLog, LogError> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("atabey")
        .filename_suffix("log")
        .build(dir)
        .map_err(|error| LogError::Setup {
            path: dir.to_path_buf(),
            reason: error.to_string(),
        })?;
    let (writer, guard) = tracing_appender::non_blocking(appender);

    let builder = fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false);
    let dispatch = match format {
        LogFormat::Text => Dispatch::new(builder.finish()),
        LogFormat::Json => Dispatch::new(builder.json().finish()),
    };
    Ok(AppLog { dispatch, guard })
}

/// Install the app log as the process-wide `tracing` subscriber. The level filter comes from
/// `RUST_LOG`, defaulting to `info`. Hold the returned guard until exit so buffered lines are
/// flushed.
///
/// # Errors
/// Returns [`LogError`] if the file can't be created or a subscriber is already installed.
pub fn init_app_log(dir: &Path, format: LogFormat) -> Result<WorkerGuard, LogError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let AppLog { dispatch, guard } = app_log(dir, format, filter)?;
    tracing::dispatcher::set_global_default(dispatch).map_err(|_| LogError::AlreadyInstalled)?;
    Ok(guard)
}
