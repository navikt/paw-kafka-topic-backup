use log::LevelFilter;
use log4rs::Config;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::json::JsonEncoder;

pub fn init_log() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(JsonEncoder::new()))
        .build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .logger(Logger::builder().build("app::paw-kafka-topic-backup", LevelFilter::Info))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config).unwrap();
}
