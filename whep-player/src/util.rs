use tracing::*;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{
    Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn init_logger(verbose: u8) -> Result<WorkerGuard, std::io::Error> {
    let level_filter = match verbose {
        0 => LevelFilter::WARN,
        1 => LevelFilter::INFO,
        2 => LevelFilter::DEBUG,
        3.. => LevelFilter::TRACE,
    };

    // 控制台日志 (pretty)
    let console_layer = fmt::layer()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_writer(std::io::stderr)
        .with_filter(level_filter);

    // 文件日志
    // * 按小时轮转
    // * 最大存储 7 天
    let file_appender = rolling::RollingFileAppender::builder()
        .rotation(rolling::Rotation::HOURLY)
        .filename_prefix("whep-player")
        .max_log_files(7 * 24)
        .build("logs")
        .expect("Create log file failed");
    // _guard 用于确保文件句柄在程序退出时被关闭, 可以顺利 dump 所有日志
    // 必须将 _guard 返回, 否则后台的日志记录线程会直接退出导致日志不会被记录到文件中
    let (non_blocking_appender, _guard) = non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_writer(non_blocking_appender)
        .with_filter(LevelFilter::TRACE);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("Init logger successfully!");

    return Ok(_guard);
}
