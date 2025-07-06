// use std::fs;
// use std::path::Path;
// use tracing::*;
// use tracing_appender::{non_blocking, rolling};
// use tracing_subscriber::{
//     filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, Registry,
// };

// /**
//  * - 同时输出到控制台和文件
//  * - 控制台开启 pretty, 文件使用普通格式
//  * - 日志文件按小时分割, 最大存储时间为 7 天
//  */
// pub fn init_logger(verbose: u8) {
// let level_filter = match verbose {
//     0 => Level::WARN,
//     1 => Level::INFO,
//     2 => Level::DEBUG,
//     3.. => Level::TRACE,
// };

// 控制台日志 (开启 pretty)
// let console_layer = fmt::layer()
//     .pretty()
//     .with_file(true)
//     .with_line_number(true)
//     .with_target(false)
//     .with_writer(std::io::stderr);

// let console_layer = fmt::layer()
//     .with_file(true)
//     .with_line_number(true)
//     .with_target(false)
//     .with_level(true)
//     .with_filter(LevelFilter::INFO);

// let log_file_path = "logs/bitwhip.log";
// let log_dir = Path::new(log_file_path).parent().unwrap();
// let log_file = fs::File::options()
//     .append(true)
//     .create(true)
//     .open(log_file_path)
//     .expect("Failed to open log file");

// fs::create_dir_all(log_dir).expect("Failed to create log directory");

// tracing_subscriber::fmt()
//     .pretty() // 启用多行美化格式
//     .with_file(true)
//     .with_line_number(true)
//     .with_target(false)
//     .with_max_level(level_filter)
//     .with_writer(log_file)
//     .init();

// info!("Init logger successfully!");
// }
