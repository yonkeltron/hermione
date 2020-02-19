use colored::*;
use slog::*;
use slog_async::Async;
use slog_json::Json;
use slog_term::{
    CountingWriter, Decorator, FullFormat, RecordDecorator, Serializer, TermDecorator,
};

use std::io;
use std::io::Write;

struct HermioneFormat<D>
where
    D: Decorator,
{
    decorator: D,
}

impl<D> Drain for HermioneFormat<D>
where
    D: Decorator,
{
    type Ok = ();
    type Err = io::Error;

    fn log(
        &self,
        record: &Record<'_>,
        values: &OwnedKVList,
    ) -> std::result::Result<<Self as Drain>::Ok, <Self as Drain>::Err> {
        self.format_full(record, values)
    }
}

impl<D> HermioneFormat<D>
where
    D: Decorator,
{
    pub fn new(d: D) -> Self {
        HermioneFormat { decorator: d }
    }

    fn format_full(&self, record: &Record, values: &OwnedKVList) -> io::Result<()> {
        self.decorator.with_record(record, values, |decorator| {
            let comma_needed = print_msg(decorator, record)?;
            {
                let mut serializer = Serializer::new(decorator, comma_needed, true);

                record.kv().serialize(record, &mut serializer)?;

                values.serialize(record, &mut serializer)?;

                serializer.finish()?;
            }

            decorator.start_whitespace()?;
            writeln!(decorator)?;

            decorator.flush()?;

            Ok(())
        })
    }
}

fn print_msg(mut rd: &mut dyn RecordDecorator, record: &Record) -> io::Result<bool> {
    rd.start_msg()?;
    let mut count_rd = CountingWriter::new(&mut rd);
    let message = format!("{}", record.msg());
    let check_message = message.to_lowercase().clone();
    let formated_message = pretty_format_rules(&check_message, &message);
    write!(count_rd, "{}", formated_message)?;
    Ok(count_rd.count() != 0)
}

fn pretty_format_rules(msg: &str, og_message_format: &str) -> String {
    let result = if msg.starts_with("successfully") {
        format!("⎿ -> {}", og_message_format.green().bold())
    } else if msg.starts_with("->") {
        format!("{}", og_message_format.replace("->", "│ ->"))
    } else if msg.ends_with("action") {
        format!("\n{} 🚀\n", og_message_format.magenta().underline())
    } else if msg.contains("success") {
        format!("{}", og_message_format.green().bold())
    } else if msg.contains("fail") {
        format!("{}", og_message_format.red().bold())
    } else if msg.contains("->") {
        format!("{}", og_message_format.replace("->", "\n⎿ ->"))
    } else {
        String::from(og_message_format)
    };
    result
}

pub fn create_default_logger() -> Logger {
    create_logger("human", Level::Debug)
}

pub fn create_logger(format: &str, level: Level) -> Logger {
    let decorator = TermDecorator::new().build();

    match format {
        "human" => {
            let drain = HermioneFormat::new(decorator).fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            let drain = Async::new(drain).build().fuse();
            Logger::root(drain, o!())
        }
        "log" => {
            let drain = FullFormat::new(decorator).build().fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            let drain = Async::new(drain).build().fuse();
            Logger::root(drain, o!())
        }
        "prettyjson" => {
            let drain = Json::new(std::io::stdout())
                .set_pretty(true)
                .add_default_keys()
                .build()
                .fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            let drain = Async::new(drain).build().fuse();
            Logger::root(drain, o!())
        }
        "json" => {
            let drain = Json::new(std::io::stdout())
                .add_default_keys()
                .build()
                .fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            let drain = Async::new(drain).build().fuse();
            Logger::root(drain, o!())
        }
        _ => {
            let drain = HermioneFormat::new(decorator).fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            let drain = Async::new(drain).build().fuse();
            Logger::root(drain, o!())
        }
    }
}
