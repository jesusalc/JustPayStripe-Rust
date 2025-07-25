use super::*;

// struct MyLogger;

// impl log::Log for MyLogger {
//     fn enabled(&self, metadata: &Metadata) -> bool {
//         // Example filter: Enable only Info level messages for a specific module
//         metadata.level() == Level::Info && metadata.target().ends_with("my_module")
//     }

//     fn log(&self, record: &Record) {
//         if self.enabled(record.metadata()) {
//             println!("[TRIGGERER] {} - {}", record.level(), record.args());
//         }
//     }

//     fn flush(&self) {}
// }

// fn init_custom_logger() -> Result<(), SetLoggerError> {
//     set_boxed_logger(Box::new(MyLogger))
//         .map(|()| set_max_level(LevelFilter::Info))
// }

pub fn setup_logger() {
    let this_script_name = Path::new(&stdenv::args().next().unwrap_or_default())
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_owned();
    let this_script_name_with_version = match stdenv::var("RUST_LOG") {
        Ok(val) if val.to_lowercase() == "trace" => "T".to_string().dimmed(),
        _ => format!("{}_{}", this_script_name, VERSION).dimmed(),
    };
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(move |buf, record| {
            let level = match record.level() {
                log::Level::Error => format!("{}", record.level()).red(),
                log::Level::Warn => format!(" {}", record.level()).yellow(),
                log::Level::Info => format!(" {}", record.level()).green(),
                log::Level::Debug => format!("{}", record.level()).blue(),
                log::Level::Trace => format!("{}", record.level()).purple(),
            };
            let file = record.file().unwrap_or("unknown");
            let line = record.line().map_or(0, |l| l);

            match &*this_script_name_with_version {
                "T" => writeln!(buf, "{}:{} {}: {}", file, line, level, record.args()),
                // format!("{}", Local::now().format("%Y%m%d %H:%M:%S")).purple(),
                _ => writeln!(
                    buf,
                    "{}:{} [{} {}]{}: {}",
                    file,
                    line,
                    format!("{}", this_script_name_with_version).purple(),
                    format!("{}", Local::now().format("%Y%m%d %H:%M:%S")).purple(),
                    level,
                    record.args()
                ), // format!("{}", ".").purple(),
            }
        })
        .init();
    info!("test");
    // Log some messages using the `log` crate's macros
    trace!("test");
    debug!("test");
    warn!("test");
    error!("test");
    info!("Logger initialized");
}
