pub fn init_logger() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_max_level(tracing::Level::DEBUG)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::*;

    // RUST_LOG=debug cargo test -p util test_logger
    // RUST_LOG=info cargo test -p util test_logger
    // RUST_LOG=warn cargo test -p util test_logger
    #[test]
    fn test_logger() {
        init_logger();
        info!("This is an info message");
        warn!("This is a warning");
        error!("This is an error");
        debug!("This is a debug message");
        trace!("This is a trace message");
    }
}
