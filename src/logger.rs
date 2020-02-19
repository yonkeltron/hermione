use slog::{o, Drain, Level, Logger};
use slog_async::Async;
use slog_json::Json;
use slog_term::{FullFormat, TermDecorator};

#[cfg(test)]
pub fn create_default_logger() -> Logger {
    create_logger("human", Level::Trace)
}

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