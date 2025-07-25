use actix_cors::Cors;
use actix_web::{
    http::{header, uri::Uri, StatusCode},
    // middleware::Logger,
    middleware::Logger as ActixLogger,
    web,
    App,
    HttpResponse,
    HttpServer,
    Responder,
    HttpRequest,
    // rt::System
};

use csv::ReaderBuilder;
use dotenv::dotenv;
use env_logger::{Builder, Env};
use lazy_static::lazy_static;
// use log::{error, info, warn, LevelFilter, Record};
// use log::{error, info, warn, trace, debug, Record, Level, Metadata, LevelFilter, SetLoggerError, set_boxed_logger, set_max_level};
use ansi_term::Colour::{Blue, Green, Purple, Red, Yellow};
use log::{debug, error, info, trace, warn, Level};
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::{
    env,
    fs::File,
    io::{
        // self,
        Read, BufRead, BufReader, Error as IOError, ErrorKind, Write},
    path::Path,
    collections::HashMap,
    // fmt::format!,
    process::{id as process_id, self, exit, Command, Stdio},
    sync::Mutex,
};

use regex::Regex;
use futures::StreamExt as _; // Make sure you have the futures crate
use chrono::Local;
use colored::*;

#[derive(Serialize)]
struct ApiResponse {
    status: String,
    command: String,
    pid: u32, // This should be u32 to match the return type of std::process::id()
    data: String, // Or Vec<String> if you expect multiple lines
    msg: String,  // Or Vec<String> if you expect multiple lines
}
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

// fn load_cors_origins(path: &str) -> io::Result<Vec<String>> {
//     let file = File::open(path)?;
//     let buf = io::BufReader::new(file);
//     buf.lines().collect()
// }
const VERSION: &str = env!("CARGO_PKG_VERSION");

lazy_static! {
    static ref PIDS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
}

fn load_env_file() {
    // Get the current directory
    let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    // Build the expected .env file path
    let env_path = current_dir.join(".env");

    // Try to load the .env file
    if dotenv().is_err() {
        // Log an error message with the expected path
        error!(
            ".env file not found. Expected it at: {}",
            env_path.display()
        );
    } else {
        info!(
            ".env loading at: {}",
            env_path.display()
        );
    }
}

fn check_env_cors() {
    let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env_cors_path = current_dir.join(".env_cors");

    if env_cors_path.exists() {
        info!(".env_cors file found at: {}", env_cors_path.display());
    } else {
        error!(".env_cors file not found. Expected it at: {}", env_cors_path.display());
    }
}

fn load_env_var(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        if default == "/home/agrivero" {
            env::var("HOME").unwrap_or_else(|_| "/home".to_string())
        } else {
            default.to_string()
        }
    })
}

// fn load_env_var(key: &str, default: &str) -> String {
//     env::var(key).unwrap_or_else(|_| default.to_string())
// }

fn sanitize_input(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9 _\-.'$%öüä]").unwrap();
    re.replace_all(input, "").to_string()
}

async fn execute_command(command: &str) -> (StatusCode, String, u32) {
    let child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(child) => {
            let pid = child.id();
            let output = child.wait_with_output().expect("failed to wait on child");

            let mut pids = PIDS.lock().unwrap();
            pids.push(pid);

            if output.status.success() {
                (
                    StatusCode::OK,
                    String::from_utf8_lossy(&output.stdout).to_string(),
                    pid,
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from_utf8_lossy(&output.stderr).to_string(),
                    pid,
                )
            }
        }
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string(), 0),
    }
}


// // one level json data deeper 1 level only
// async fn reports() -> impl Responder {
//     let pwd = env::current_dir().unwrap_or_else(|_| ".".into());
//     info!("Current working directory: {:?}", pwd.display());

//     let path = pwd.join("reports.csv");
//     let file = match File::open(&path) {
//         Ok(file) => file,
//         Err(_) => {
//             error!("Failed to find reports.csv at {:?}", path.display());
//             return HttpResponse::BadRequest().json(ApiResponse {
//                 status: "error".to_string(),
//                 pid: std::process::id(),
//                 data: "".to_string(),
//                 msg: format!("Failed to find reports.csv at {:?}", path.display()),
//             });
//         }
//     };

//     let mut reader = ReaderBuilder::new().delimiter(b';').from_reader(file);
//     let headers = match reader.headers() {
//         Ok(headers) => headers.clone(),
//         Err(e) => {
//                 error!("Failed to read headers reports.csv at {:?} error:{:?}", path.display(), e);
//                 return HttpResponse::InternalServerError().json(ApiResponse {
//                 status: "error".to_string(),
//                 pid: std::process::id(),
//                 data: "".to_string(),
//                 msg: format!("Failed to read headers reports.csv at {:?}, error:{:?}", path.display(), e),
//             })
//         }
//     };

//     // let mut data = Vec::new();

//     let mut records = vec![];
//     let mut errors = Vec::new();
//     let mut row_count = 0;
//     let mut error_count = 0;

//     for (i, result) in reader.records().enumerate() {
//         match result {
//             Ok(record) => {
//                 if record.iter().any(|x| !x.trim().is_empty()) {
//                     let record_map: serde_json::Map<String, serde_json::Value> = headers
//                         .iter()
//                         .zip(record.iter())
//                         .map(|(h, v)| (h.to_string(), json!(v)))
//                         .collect();

//                     records.push(json!(record_map));
//                 } else {
//                     error_count += 1;
//                     error!("Invalid row at line {}: Empty or whitespace-only", i + 2);
//                     errors.push(format!("Invalid row at line {}: Empty or whitespace-only", i + 2));  // i + 2 to account for header row and zero-index
//                 }
//                 row_count += 1;
//             },
//             Err(e) => {
//                 error_count += 1;
//                 error!("Error reading row at line {}: {}", i + 2, e);
//                 errors.push(format!("Error reading row at line {}: {}", i + 2, e));
//             }
//         }
//     }


//     if row_count == 0 || row_count == error_count {
//         error!("All rows failed or file is empty.");
//         return HttpResponse::InternalServerError().json(ApiResponse {
//             status: "error".to_string(),
//             pid: std::process::id(),
//             data: "".to_string(),
//             msg: "All rows failed or file is empty.".to_string(),
//         });
//     }

//     if error_count > 0 {
//         let response_data = json!({
//             "success_count": records.len(),
//             "errors_count": errors.len(),
//             "records": records,
//         });
//         let response = ReportsResponse::new(
//             "partial_success".to_string(),
//             response_data,
//             errors.join("\n"),
//         );

//         // Return HttpResponse with JSON
//         HttpResponse::Ok().json(response);
//     }
//     let response_data = json!({
//         "success_count": records.len(),
//         "errors_count": errors.len(),
//         "records": records,
//     });
//     let response = ReportsResponse::new(
//         "success".to_string(),
//         response_data,
//         "".to_string(),
//     );
//     HttpResponse::Ok().json(response)
// }

// Route working with plan test serving
// async fn reports() -> impl Responder {
//     let pwd = env::current_dir().unwrap_or_else(|_| ".".into());
//     info!("Current working directory: {:?}", pwd.display());

//     let path = pwd.join("reports.csv");
//     let file = match File::open(&path) {
//         Ok(file) => file,
//         Err(_) => {
//             error!("Failed to find reports.csv at {:?}", path.display());
//             return HttpResponse::BadRequest().json(ApiResponse {
//                 status: "error".to_string(),
//                 pid: std::process::id(),
//                 data: "".to_string(),
//                 msg: format!("Failed to find reports.csv at {:?}", path.display()),
//             });
//         }
//     };

//     let mut reader = ReaderBuilder::new().delimiter(b';').from_reader(file);
//     let mut data = Vec::new();
//     let mut errors = Vec::new();
//     let mut row_count = 0;
//     let mut error_count = 0;

//     for (i, result) in reader.records().enumerate() {
//         match result {
//             Ok(record) => {
//                 let row: Vec<String> = record.iter().map(String::from).collect();
//                 if row.iter().any(|x| !x.trim().is_empty()) {
//                     data.push(row.join(", "));  // Join the columns of the row into a single string
//                 } else {
//                     error_count += 1;
//                     errors.push(format!("Invalid row at line {}: {:?}", i + 1, row.join(", ")));
//                 }
//                 row_count += 1;
//             },
//             Err(e) => {
//                 error_count += 1;
//                 errors.push(format!("Error reading a row at line {}: {}", i + 1, e));
//                 error!("Error reading a row at line {}: {}", i + 1, e);
//             }
//         }
//     }

//     let data = data.join("\n"); // Join all valid rows into a single string separated by new lines
//     let msg = errors.join("\n"); // Join all error messages into a single string separated by new lines

//     if row_count == 0 || row_count == error_count {
//         return HttpResponse::InternalServerError().json(ApiResponse {
//             status: "error".to_string(),
//             pid: std::process::id(),
//             data: "".to_string(),
//             msg: "All rows failed or file is empty.".to_string(),
//         });
//     }

//     if error_count > 0 {
//         HttpResponse::Ok().json(ApiResponse {
//             status: "partial_success".to_string(),
//             pid: std::process::id(),
//             data,
//             msg,
//         })
//     } else {
//         HttpResponse::Ok().json(ApiResponse {
//             status: "success".to_string(),
//             pid: std::process::id(),
//             data,
//             msg: "".to_string(),
//         })
//     }
// }
// async fn reports() -> impl Responder {
//     let pwd = env::current_dir().unwrap_or_else(|_| ".".into());
//     info!("Current working directory: {:?}", pwd.display());

//     let path = pwd.join("reports.csv");
//     let file = match File::open(&path) {
//         Ok(file) => file,
//         Err(_) => {
//             error!("Failed to find reports.csv at {:?}", path.display());
//             return HttpResponse::BadRequest().json(ApiResponse2 {
//                 status: "error".to_string(),
//                 pid: std::process::id(),
//                 data: vec![],
//                 msg: vec![format!("Failed to find reports.csv at {:?}", path.display())],
//             });
//         }
//     };

//     let mut reader = ReaderBuilder::new().delimiter(b';').from_reader(file);
//     let mut data = Vec::new();
//     let mut msg = Vec::new();
//     let mut row_count = 0;
//     let mut error_count = 0;

//     for (i, result) in reader.records().enumerate() {
//         match result {
//             Ok(record) => {
//                 let row: Vec<String> = record.iter().map(String::from).collect();
//                 if row.iter().any(|x| !x.trim().is_empty()) {
//                     data.push(row);
//                 } else {
//                     error_count += 1;
//                     msg.push(format!("Invalid row at line {}: {:?}", i + 1, row));
//                 }
//                 row_count += 1;
//             },
//             Err(e) => {
//                 error_count += 1;
//                 msg.push(format!("Error reading a row at line {}: {}", i + 1, e));
//                 error!("Error reading a row at line {}: {}", i + 1, e);
//             }
//         }
//     }

//     if row_count == 0 || row_count == error_count {
//         return HttpResponse::InternalServerError().json(ApiResponse2 {
//             status: "error".to_string(),
//             pid: std::process::id(),
//             data: vec![],
//             msg: vec!["All rows failed or file is empty.".to_string()],
//         });
//     }

//     if error_count > 0 {
//         HttpResponse::Ok().json(ApiResponse2 {
//             status: "partial_success".to_string(),
//             pid: std::process::id(),
//             data,
//             msg,
//         })
//     } else {
//         HttpResponse::Ok().json(ApiResponse2 {
//             status: "success".to_string(),
//             pid: std::process::id(),
//             data,
//             msg: vec![],
//         })
//     }
// }

// async fn reports() -> impl Responder {
//     let pwd = env::current_dir().unwrap_or_else(|_| ".".into());
//     info!("Current working directory: {:?}", pwd.display());

//     let path = pwd.join("reports.csv");
//     let file = match File::open(&path) {
//         Ok(file) => file,
//         Err(_) => {
//             error!("Failed to find reports.csv at {:?}", path.display());
//             return HttpResponse::BadRequest().json(ApiResponse {
//                 status: "error".to_string(),
//                 pid: std::process::id(),
//                 data: format!(""),
//                 msg: format!("Failed to find reports.csv at {:?}", path.display()),
//             });
//         }
//     };

//     let mut reader = ReaderBuilder::new().delimiter(b';').from_reader(file);
//     let mut data = Vec::new();
//     let mut msg = Vec::new();
//     let mut has_valid_rows = false;

//     for result in reader.records() {
//         match result {
//             Ok(record) => {
//                 let row: Vec<String> = record.iter().map(String::from).collect();
//                 if row.iter().any(|x| !x.trim().is_empty()) { // Ignore empty lines
//                     data.push(row);
//                     has_valid_rows = true;
//                 }
//             },
//             Err(e) => {
//                 msg.push(format!("Error reading a row: {}", e));
//                 error!("Error reading a row: {}", e);
//             }
//         }
//     }

//     if !has_valid_rows {
//         return HttpResponse::BadRequest().json(ApiResponse {
//             status: "error".to_string(),
//             pid: std::process::id(),
//             data: format!(""),
//             msg: format!(""),
//         });
//     }

//     HttpResponse::Ok().json(ApiResponse {
//         status: "success".to_string(),
//         pid: std::process::id(),
//         data: format!(""),
//         msg: format!(""),
//     })
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| {
//         App::new()
//             .service(web::resource("/reports").route(web::get().to(reports)))
//     })
//     .bind("127.0.0.1:8081")?
//     .run()
//     .await
// }

// async fn reports() -> impl Responder {
//     // Command::new("sudo").arg("reboot").output().expect("failed to execute process");
//     // HttpResponse::Ok().body("System reboot triggered")
//     let (status, output, pid) =
//         execute_command("/home/agrivero/_/work/agrivero/projects/vero2/reports.sh").await;
//     HttpResponse::build(status).json(json!({
//         "status": status.as_u16(),
//         "pid": pid,
//         "data": output,
//         "msg": if status == StatusCode::OK { "" } else { "Error executing command" },
//     }))
// }

// async fn run_script(command: &str) -> impl Responder {
async fn run_script(command: &str) -> impl Responder {
    let (status, output, pid) = execute_command(command).await;
    HttpResponse::build(status).json(ApiResponse {
        command: command.to_string(),
        status: status.to_string(),
        pid,
        data: output,
        msg: if status == StatusCode::OK {
            "".to_string()
        } else {
            "Error executing command".to_string()
        },
    })
} // end run_script
// } // end run_script

// macro_rules! define_action {
macro_rules! define_action {
    ($fn_name:ident, $script_name:expr) => {
        async fn $fn_name() -> impl Responder {
            let project_path = load_env_var("PROJECT_PATH", "/default/path");
            let command = format!("{}/{}", project_path, $script_name);
            run_script(&command).await
        }
    };
} // end define_action
// } // end define_action

// define_action!(arduino_start, "arduino_start.sh");
define_action!(arduino_start, "arduino_start.sh");
// define_action!(arduino_stop, "arduino_stop.sh");
define_action!(arduino_stop, "arduino_stop.sh");
// define_action!(arduino_error, "arduino_error.sh");
define_action!(arduino_error, "arduino_error.sh");
// define_action!(capture_start, "capture_start.sh");
define_action!(capture_start, "capture_start.sh");
define_action!(capture_kill, "capture_kill.sh");
define_action!(capture_release, "capture_release.sh");
define_action!(analysis_will_start, "analysis_will_start.sh");
define_action!(analysis_will_stop, "analysis_will_stop.sh");
define_action!(analysis_vero_start, "analysis_vero_start.sh");
define_action!(analysis_vero_stop, "analysis_vero_stop.sh");
// define_action!(action1, "action1.sh");
define_action!(action1, "action1.sh");
// define_action!(action2, "action2.sh");
define_action!(action2, "action2.sh");
// define_action!(action3, "action3.sh");
define_action!(action3, "action3.sh");
// define_action!(action4, "action4.sh");
define_action!(action4, "action4.sh");
// define_action!(action5, "action5.sh");
define_action!(action5, "action5.sh");
// define_action!(action6, "action6.sh");
define_action!(action6, "action6.sh");
// define_action!(action7, "action7.sh");
define_action!(action7, "action7.sh");
// define_action!(action8, "action8.sh");
define_action!(action8, "action8.sh");

async fn reboot() -> impl Responder {
    let command = "systemctl reboot";
    let (status, output, pid) = execute_command(command).await;
    HttpResponse::build(status).json(ApiResponse {
        command: command.to_string(),
        status: status.to_string(),
        pid: pid,
        data: output,
        msg: if status == StatusCode::OK { "".to_string() } else { "Error executing command".to_string() },
    })
}


fn normalize_header(header: &str) -> String {
    use unidecode::unidecode;
    let header = unidecode(header); // Transliterate Unicode to ASCII
    let header = header.to_lowercase(); // Convert to lowercase
    let header = header.replace(|c: char| !c.is_alphanumeric() && c != '_', " "); // Replace non-alphanumeric characters except _ with space
    let header = header.split_whitespace().collect::<Vec<_>>().join("_"); // Convert spaces to underscores and remove extra ones
    header
}
#[derive(Serialize, Deserialize)]
struct ReportsResponse<T> where T: Serialize {
    command: String,
    status: String,
    pid: u32,
    data: T,
    msg: String,
}

impl<T> ReportsResponse<T> where T: Serialize {
    pub fn new(command: String, status: String, data: T, msg: String) -> ReportsResponse<T> {
        ReportsResponse {
            command,
            status,
            pid: process::id(),
            data,
            msg,
        }
    }
}

// reports that clear headers and deeper more json data
async fn reports() -> impl Responder {
    let pwd = env::current_dir().unwrap_or_else(|_| ".".into());
    info!("Current working directory: {:?}", pwd.display());

    let path = pwd.join("reports.csv");
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => {
            error!("Failed to find reports.csv at {:?}", path.display());
            return HttpResponse::BadRequest().json(ApiResponse {
                status: "error".to_string(),
                command: format!("{:?}", path.display()),
                pid: std::process::id(),
                data: "".to_string(),
                msg: format!("Failed to find reports.csv at {:?}", path.display()),
            });
        }
    };

    let mut reader = ReaderBuilder::new().delimiter(b';').from_reader(file);
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(e) => {
                error!("Failed to read headers reports.csv at {:?} error:{:?}", path.display(), e);
                return HttpResponse::InternalServerError().json(ApiResponse {
                status: "error".to_string(),
                command: "".to_string(),
                pid: std::process::id(),
                data: "".to_string(),
                msg: format!("Failed to read headers reports.csv at {:?}, error:{:?}", path.display(), e),
            })
        }
    };

    // let mut data = Vec::new();

    let mut records = vec![];
    let mut errors = Vec::new();
    let mut row_count = 0;
    let mut error_count = 0;

    for (i, result) in reader.records().enumerate() {
        match result {
            Ok(record) => {
                if record.iter().any(|x| !x.trim().is_empty()) {
                    let record_map: serde_json::Map<String, serde_json::Value> = headers
                        .iter()
                        .zip(record.iter())
                        .map(|(h, v)| {
                            let key = normalize_header(h);
                            let value = if v.contains('|') {
                                json!(v.split('|').map(|s| s.trim()).collect::<Vec<_>>())
                            } else {
                                json!(v.trim())
                            };
                            (key, value)
                        })
                        .collect();
                    records.push(json!(record_map));
                } else {
                    error_count += 1;
                    error!("Invalid row at line {}: Empty or whitespace-only", i + 2);
                    errors.push(format!("Invalid row at line {}: Empty or whitespace-only", i + 2));  // i + 2 to account for header row and zero-index
                }
                row_count += 1;
            },
            Err(e) => {
                error_count += 1;
                error!("Error reading row at line {}: {}", i + 2, e);
                errors.push(format!("Error reading row at line {}: {}", i + 2, e));
            }
        }
    }


    if row_count == 0 || row_count == error_count {
        error!("All rows failed or file is empty.");
        return HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_string(),
            command: "".to_string(),
            pid: std::process::id(),
            data: "".to_string(),
            msg: "All rows failed or file is empty.".to_string(),
        });
    }

    let response_data = json!({
        "success_count": records.len(),
        "errors_count": errors.len(),
        "records": records,
    });
    let command = "reports.csv".to_string();
    let status = if error_count > 0 { "partial_success" } else { "success" };
    let response = ReportsResponse::new(
        command.to_string(),
        status.to_string(),
        response_data,
        errors.join("\n"),
    );

    HttpResponse::Ok().json(response)
}

fn load_and_validate_cors_origins(path: &str) -> Result<Vec<String>, IOError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let mut origins = Vec::new();
    let mut all_lines_failed = true;

    for line in buf_reader.lines() {
        let line = line?;
        match line.parse::<Uri>() {
            Ok(_) => {
                origins.push(line);
                all_lines_failed = false;
            }
            Err(e) => {
                warn!("Invalid URI in CORS configuration: {}", e);
            }
        }
    }

    if all_lines_failed {
        return Err(IOError::new(
            ErrorKind::InvalidData,
            "All CORS lines failed validation.",
        ));
    }

    Ok(origins)
}

fn setup_logger() {

    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let this_script_name = Path::new(&env::args().next().unwrap_or_default())
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_owned();
            let this_script_name_with_version = format!("{}_{}", this_script_name, VERSION);
    
            let level = record.level();
            let color = match level {
                Level::Error => format!("{}", Red.paint(level.to_string())),
                Level::Warn =>  format!(" {}", Yellow.paint(level.to_string())),
                Level::Info =>  format!(" {}", Green.paint(level.to_string())),
                Level::Debug => format!("{}", Blue.paint(level.to_string())),
                Level::Trace => format!("{}", Purple.paint(level.to_string())),
            };
            // Simplified format to avoid redundancy
            writeln!(buf,
                "[{} {}]{}: {}",
                format!("{}", this_script_name_with_version).dimmed(),
                Local::now().format("%Y%m%d %H:%M:%S").to_string().dimmed(),
                color,
                record.args())
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



async fn list_routes() -> impl Responder {
    dotenv().ok();

    let target_port = load_env_var("PORT", "8081");
    let target_host = load_env_var("HOST", "127.0.0.1");

    let host = format!("{}://{}:{}", "http", target_host, target_port);
    let routes = vec![
        "/arduino/start",
        "/arduino/stop",
        "/arduino/error",
        "/capture/start",
        "/capture/kill",
        "/camera/release",
        "/analysis/will/start",
        "/analysis/will/stop",
        "/analysis/vero/start",
        "/analysis/vero/stop",
        "/reboot",
        "/reports",
        "/action1",
        "/action2",
        "/action3",
        "/action4",
        "/action5",
        "/action6",
        "/action7",
        "/action8",
    ];

    let data: Vec<_> = routes
        .iter()
        .map(|route| {
            json!({
                "relative_url": route,
                "absolute_url": format!("{}{}", host, route),
            })
        })
        .collect();

    HttpResponse::Ok().json(json!({ "data": data }))
}

async fn execute_script(req: HttpRequest, mut body: web::Payload) -> impl Responder {
    let action = sanitize_input(req.match_info().query("tail"));
    let project_path = load_env_var("PROJECT_PATH", "/home/agrivero");



    // Read the body
    let mut body_bytes = web::BytesMut::new();
    while let Some(chunk) = body.next().await {
        match chunk {
            Ok(bytes) => body_bytes.extend_from_slice(&bytes),
            Err(_) => return HttpResponse::BadRequest().json(ApiResponse {
                status: "error".to_owned(),
                command: "".to_owned(),
                pid: std::process::id(),
                data: "".to_owned(),
                msg: "Invalid request body".to_owned(),
            }),
        }
    }
    let body_str = sanitize_input(&String::from_utf8(body_bytes.to_vec()).unwrap_or_default());

    // Sanitize the action string
    let sanitized_action = Regex::new(r"[^a-z0-9]+").unwrap()
        .replace_all(&action, "_")
        .to_string();
    let sanitized_action = Regex::new(r"_+").unwrap()
        .replace_all(&sanitized_action, "_")
        .to_string();

    // Parse query parameters
    let query_params = web::Query::<HashMap<String, String>>::from_query(req.query_string()).unwrap();
    let mut args: Vec<String> = vec![];

    // Iterate over the query parameters
    for (key, value) in query_params.iter() {
        let formatted_value = if value.contains(' ') {
            format!(r#"{}="{}""#, key, value)
        } else {
            format!(r#"{}={}"#, key, value)
        };
        args.push(formatted_value);
    }

    // Define the file paths
    let bash_script = format!("{}/{}.bash", project_path, sanitized_action);
    let sh_script = format!("{}/{}.sh", project_path, sanitized_action);
    // Attempt to find the script and construct the command
    let bash_script_to_run = std::path::Path::new(&bash_script);
    let sh_script_to_run = std::path::Path::new(&sh_script);

    // let this_script_relative_path = std::env::args().next().unwrap_or_default();
    // let this_script_name = std::path::Path::new(&this_script_relative_path)
    //   .file_name()
    //   .unwrap_or_default()
    //   .to_str()
    //   .unwrap_or_default()
    //   .to_owned();
    let this_script_absolute_pathbuf = std::env::current_exe().expect("Failed to get the current executable path");
    let this_script_absolute_path = std::path::Path::new(&this_script_absolute_pathbuf);

    // info!("\x1b[01;35m # THIS SCRIPT NAME\x1b[38;5;93m:\x1b[38;5;1m {}", this_script_name);
    // info!("\x1b[01;35m # THIS SCRIPT RELATIVE PATH\x1b[38;5;93m:\x1b[38;5;1m {}", this_script_relative_path);
    // info!("\x1b[01;35m # THIS SCRIPT ABSOLUTE PATH\x1b[38;5;93m:\x1b[38;5;1m {:?}", this_script_absolute_path);
    info!("\x1b[01;35m checking PATH\x1b[38;5;93m:\x1b[38;5;1m {:?}", this_script_absolute_path);
    let script_to_run = if bash_script_to_run.exists() {
        info!("\x1b[01;35m found script\x1b[38;5;93m:\x1b[38;5;1m {:?}", bash_script_to_run);
        bash_script_to_run
    } else if sh_script_to_run.exists() {
        info!("\x1b[01;35m found script\x1b[38;5;93m:\x1b[38;5;1m {:?}", sh_script_to_run);
        sh_script_to_run
    } else {
        error!("\x1b[01;35m No scripts found in path:\x1b[38;5;93m:\x1b[38;5;1m{:?} looked for bash: {:?} looked for sh:{:?}", this_script_absolute_path, bash_script_to_run, sh_script_to_run);
        this_script_absolute_path
    };
    if script_to_run == this_script_absolute_path {
        return HttpResponse::NotFound().json(ApiResponse {
            status: "error".to_string(),
            command: format!("bash {:?}  sh {:?} ",bash_script_to_run,  sh_script_to_run),
            pid: std::process::id(),
            data: format!(""),
            msg: format!("No script found for the requested action bash {:?}  sh {:?} ",bash_script,  sh_script),
        });
    }
    let  script_to_run_clone1 = script_to_run;
    let  script_to_run_clone2 = script_to_run;
    let  script_to_run_clone3 = script_to_run;
    // let  script_to_run_clone4 = script_to_run.clone();


    // Execute the script and provide the body_str as input
    let mut child = match Command::new(&script_to_run)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
        Ok(child) => child,
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_owned(),
            command: format!("{:?}", script_to_run),
            pid: std::process::id(),
            data: "".to_owned(),
            msg: format!("Failed to execute script {:?}", script_to_run),
        }),
    };

    // Write to stdin of the script if necessary
    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(body_str.as_bytes()).is_err() {
            return HttpResponse::InternalServerError().json(ApiResponse {
                status: "error".to_owned(),
                command: "Failed to write script stdin".to_owned(),
                pid: std::process::id(),
                data: "".to_owned(),
                msg: "Failed to write script stdin".to_owned(),
            });
        }
    }

    // // Execute the script and stream output
    // let mut child = match Command::new(script_to_run)
    //     .stdout(Stdio::piped())
    //     .stderr(Stdio::piped())
    //     .spawn() {
    //         Ok(child) => child,
    //         Err(_) => return HttpResponse::InternalServerError().json(ApiResponse {
    //             status: "error".to_string(),
    //             pid: std::process::id(),
    //             data: format!(""),
    //             msg: format!("Failed to execute script {:?}  ",script_to_run_clone4),
    //         }),
    //     };

    let mut output = String::new();
    if let Some(ref mut stdout) = child.stdout {
        stdout.read_to_string(&mut output).unwrap_or_else(|_| 0);
    }

    let status = match child.wait() {
        Ok(status) => status.code().unwrap_or_default(),
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_string(),
            command: format!("{:?}", script_to_run_clone3),
            pid: std::process::id(),
            data: format!(""),
            msg: format!("Error occurred while waiting for the script  {:?} ",script_to_run_clone3),
        }),
    };

    if status != 0 {
        let mut error_output = String::new();
        if let Some(ref mut stderr) = child.stderr {
            stderr.read_to_string(&mut error_output).unwrap_or_else(|_| 0);
        }

        return HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_string(),
            command: format!("{:?}", script_to_run_clone2),
            pid: std::process::id(),
            data: format!("{:?}", output),
            msg: format!("Error occurred while waiting for the script  {:?} ", script_to_run_clone2),
        });
    }

    HttpResponse::Ok().json(ApiResponse {
        status: "success".to_string(),
        command: format!("{:?}", script_to_run_clone1),
        pid: std::process::id(),
        data: format!("{:?}", output),
        msg: format!("Ran script  {:?} ",script_to_run_clone1),
    })
}

fn print_help() {
    let version = VERSION;
    let this_script_relative_path = env::args().next().unwrap_or_default();
    let _this_script_absolute_path = env::current_exe().expect("Failed to get the current executable path");
    let _call_from_absolute_path = env::current_dir().expect("Failed to get the current directory");
    let this_script_name = Path::new(&this_script_relative_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_owned();
    println!("{} version:{} Usage: {} ", this_script_name, version, this_script_name);
    println!("Creates a server listener in provided port. Then runs bash files or reads csv files to report. ");
    println!("No arguments besides this --help message");
    println!("Depends on optional .env file ");
    println!("Depends on optional .env_cors file");
    println!("Options:");
    println!("  --help, -h             Show this help message");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from("--help")) || args.contains(&String::from("-h")) {
        print_help();
        exit(0);
    }

    let this_script_relative_path = std::env::args().next().unwrap_or_default();
    let this_script_name = std::path::Path::new(&this_script_relative_path)
      .file_name()
      .unwrap_or_default()
      .to_str()
      .unwrap_or_default()
      .to_owned();
    let this_script_absolute_pathbuf = std::env::current_exe().expect("Failed to get the current executable path");
    let this_script_absolute_path = std::path::Path::new(&this_script_absolute_pathbuf);

    // Initialize the logger
    setup_logger();

    // Load .env .env_cors files and log error if not found
    load_env_file();
    check_env_cors();

    dotenv().ok();

    info!("\x1b[01;35m # THIS SCRIPT NAME\x1b[38;5;93m:\x1b[38;5;1m {}", this_script_name);
    info!("\x1b[01;35m # THIS SCRIPT RELATIVE PATH\x1b[38;5;93m:\x1b[38;5;1m {}", this_script_relative_path);
    info!("\x1b[01;35m # THIS SCRIPT ABSOLUTE PATH\x1b[38;5;93m:\x1b[38;5;1m {:?}", this_script_absolute_path);

    let target_port = load_env_var("PORT", "8081");
    let target_host = load_env_var("HOST", "127.0.0.1");
    let target_server = format!("{}:{}", target_host, target_port);

    // Initialize error flags
    let mut cors_failed = false;
    let mut port_failed = false;
    let mut when_errors_detected = false;
    // Attempt to load allowed origins
    let allowed_origins = load_and_validate_cors_origins(".env_cors").unwrap_or_else(|e| {
        cors_failed = true;
        error!("Failed to load .env_cors, error: {:?}", e);
        vec![]
    });

    // For debugging: print out the allowed origins
    info!("Allowed origins: {:?}", allowed_origins);

    trace!(
        "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
        when_errors_detected,
        cors_failed,
        port_failed
    );

    let cors_origins = match load_and_validate_cors_origins(".env_cors") {
        Ok(origins) => {
            info!("CORS origins loaded successfully.");
            origins
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            cors_failed = true; // Set cors_failed flag
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );

            let pwd = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
            error!(".env_cors file not found in directory: {:?}", pwd.display());
            exit(1);
        }
        Err(e) => {
            cors_failed = true; // Set cors_failed flag
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            error!("Failed to load or validate all CORS origins: {}", e);
            exit(1);
        }
    };
    // For debugging: print out the allowed origins
    info!("Allowed cors_origins: {:?}", cors_origins);

    // Check if `lsof` is available
    let lsof_available = Command::new("sh")
        .arg("-c")
        .arg("which lsof")
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);

    if !lsof_available {
        info!("`lsof` is not available. Please install `lsof` for more detailed diagnostics.");

        // Check if port 8081 is in use (simplified check)
        if std::net::TcpListener::bind(format!("{}", target_server)).is_err() {
            port_failed = true; // Set port_failed flag
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            error!("Port {} is already in use.", target_port);
            exit(52);
        }
    } else {
        // Check if port 8081 is in use
        match std::net::TcpListener::bind(format!("{}", target_server)) {
            Ok(_) => {
                // Port is free, continue with server setup...
            }
            Err(_) => {
                port_failed = true; // Set port_failed flag
                trace!(
                    "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                    when_errors_detected,
                    cors_failed,
                    port_failed
                );
                error!("Port {} is already in use.", target_port);
                if lsof_available {
                    // Attempt to identify what's using port 8081 using `lsof`
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(format!("lsof -i :{} -t -sTCP:LISTEN",target_port))
                        .output();

                    match output {
                        Ok(output) if !output.stdout.is_empty() => {
                            let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                            info!("PID using port {}: {}", target_port, pid);

                            // Optionally, get more details about the process
                            let cmd = format!("ps -o user= -o comm= -p {}", pid);
                            if let Ok(output) = Command::new("sh").arg("-c").arg(cmd).output() {
                                info!(
                                    "Process details: {}",
                                    String::from_utf8_lossy(&output.stdout)
                                );
                            }
                        }
                        _ => error!("Could not determine the process using port {}", target_port),
                    }
                }
                exit(52);
            }
        }
    }

    // Determine if any errors have been detected
    when_errors_detected = cors_failed || port_failed;
    trace!(
        "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
        when_errors_detected,
        cors_failed,
        port_failed
    );
    // Log the PID of the main server process
    let server_pid = process_id();
    info!("Server starting with PID: {}", server_pid);

    if when_errors_detected {
        // Handle the case where errors were detected
        error!("Server start-up failed due to errors.");
        // return Err(e);
        exit(1); // Exit with a generic error code
    } else {
        let server = HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_method()
                .allow_any_header()
                .supports_credentials()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
                .allowed_headers(vec![
                    header::AUTHORIZATION,
                    header::ACCEPT,
                    header::CONTENT_TYPE,
                ])
                .max_age(3600);

            trace!("1 cors: {:?}", cors);
            // let cors = cors_origins
            //     .iter()
            //     .fold(cors, |cors, origin| cors.allowed_origin(origin));
            // Dynamically add allowed origins from the .env_cors file
            // let cors = cors_origins
            //     .iter()
            //     .fold(cors, |cors, origin| cors.allowed_origin(origin));
            let cors = Cors::permissive();
            trace!("2 cors: {:?}", cors);

            App::new()
                .wrap(ActixLogger::default())
                // .wrap(Logger::default())
                .wrap(cors)
                .route("/arduino/start", web::get().to(arduino_start))
                .route("/arduino/stop", web::get().to(arduino_stop))
                .route("/arduino/error", web::get().to(arduino_error))
                .route("/capture/start", web::get().to(capture_start))
                .route("/capture/kill", web::get().to(capture_kill))
                .route("/camera/release", web::get().to(capture_release))
                .route("/analysis/will/start", web::get().to(analysis_will_start))
                .route("/analysis/will/stop", web::get().to(analysis_will_stop))
                .route("/analysis/vero/start", web::get().to(analysis_vero_start))
                .route("/analysis/vero/stop", web::get().to(analysis_vero_stop))
                .route("/reports", web::get().to(reports))
                .service(
                    web::resource("/actions/{tail:.*}")
                        .route(web::get().to(execute_script))
                        .route(web::post().to(execute_script))
                        .route(web::put().to(execute_script))
                        .route(web::patch().to(execute_script)),
                    )
                // .service(web::resource("/reports").route(web::get().to(reports)))
                .route("/action1", web::get().to(action1))
                .route("/action2", web::get().to(action2))
                .route("/action3", web::get().to(action3))
                .route("/action4", web::get().to(action4))
                .route("/action5", web::get().to(action5))
                .route("/action6", web::get().to(action6))
                .route("/action7", web::get().to(action7))
                .route("/action8", web::get().to(action8))
                .route("/reboot", web::get().to(reboot))
                .route("/actions/{tail:.*}", web::get().to(execute_script))
                .route("/actions/{tail:.*}", web::post().to(execute_script))
                .route("/list_routes", web::get().to(list_routes))
                .default_service(web::get().to(list_routes))
            })
            .bind(format!("{}", target_server))?
            .run();

        info!("Server running at http://{} ", format!("{}", target_server));
        trace!(
            "when_errors_detected: {:?} cors_failed:{:?} port_failed:{:?}",
            when_errors_detected,
            cors_failed,
            port_failed
        );

        let execution = server.await;

        // Log when the server stops
        info!("Worker stopped with PID: {}", process_id());

        // Handle server run error
        if let Err(e) = execution {
            trace!(
                "when_errors_detected: {:?} cors_failed:{:?} port_failed:{:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            error!("Failed to start the server: {:?}", e);
            return Err(e);
        } else {
            if port_failed {
                error!("Port {} is already in use.", format!("{}", target_server));

                // If determining which process uses the port is crucial, you might attempt
                // to identify and log the responsible process here (as previously discussed).

                // Exiting if the port is already in use
                exit(1);
                // } else {
                // Proceed with server setup if the port is available
                // info!("Starting server on {}", port);
                // Your server startup logic here...
            }

            Ok(())
        }
    }
}
