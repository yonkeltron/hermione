use slog::{o, Drain, Level, Logger};
use slog_async::Async;
use slog_json::Json;
use slog_term::{FullFormat, TermDecorator};

#[cfg(test)]
pub fn create_testing_logger() -> Logger {
    use std::str::FromStr;
    let log_level_env = std::env::var("HERMIONE_TEST_LOG_LEVEL");
    let log_level_string = log_level_env
        .as_ref()
        .map(String::as_str)
        .unwrap_or("Critical");
    let log_level = Level::from_str(&log_level_string).unwrap_or(Level::Critical);
    create_logger("human", log_level)
}

/// Creates an instance of a `slog` logger.
///
/// ### Arguments
///
/// * format - Format for the output, currently supports `human` `prettyjson` and `json`
/// * level - Log level for the printouts, default is `INFO`
///
/// Returns a Logger.
pub fn create_logger(format: &str, level: Level) -> Logger {
    let decorator = TermDecorator::new().build();

    let drain = match format {
        "human" => {
            let drain = FullFormat::new(decorator).build().fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            Async::new(drain).build().fuse()
        }
        "prettyjson" => {
            let drain = Json::new(std::io::stdout())
                .set_pretty(true)
                .add_default_keys()
                .build()
                .fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            Async::new(drain).build().fuse()
        }
        "json" => {
            let drain = Json::new(std::io::stdout())
                .add_default_keys()
                .build()
                .fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            Async::new(drain).build().fuse()
        }
        _ => {
            let drain = FullFormat::new(decorator).build().fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            Async::new(drain).build().fuse()
        }
    };
    Logger::root(drain, o!())
}
