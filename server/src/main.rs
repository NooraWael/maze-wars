mod server;

use chrono::Local;
use colored::*;
use log::LevelFilter;
use server::Server;

struct ColoredLogger;

impl log::Log for ColoredLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Local::now().format("%H:%M:%S%.3f");
            let level = match record.level() {
                log::Level::Error => format!("[{}]", record.level()).red(),
                log::Level::Warn => format!("[{}]", record.level()).yellow(),
                log::Level::Info => format!("[{}]", record.level()).cyan(),
                log::Level::Debug => format!("[{}]", record.level()).purple(),
                log::Level::Trace => format!("[{}]", record.level()).blue(),
            };
            let output = format!(
                "{} {} {}",
                timestamp.to_string().dimmed(),
                level,
                record.args()
            );
            if record.level() == log::Level::Error {
                eprintln!("{}", output);
            } else {
                println!("{}", output);
            }
        }
    }

    fn flush(&self) {}
}

#[tokio::main]
async fn main() {
    log::set_logger(&ColoredLogger)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();

    if let Err(e) = Server::new("0.0.0.0", 2025)
        .min_players(2)
        .max_players(10)
        .start()
        .await
    {
        log::error!("{}", e)
    }
}
