use actix_cors::Cors;
use dotenvy::*;
use justpaystripe::{
    stripe::{Charge, Customer},
    StripeClient,
};
mod logger;
use crate::logger::*;
mod cors;
use crate::cors::*;
mod env;
use crate::env::*;
use actix_web::{
    // cookie::Cookie,
    http::{
        header,
        uri::Uri,
        // StatusCode
    },
    // middleware::Logger,
    middleware::Logger as ActixLogger,
    web,
    App,
    // HttpRequest,
    // rt::System
    HttpResponse,
    HttpServer,
    Responder,
};
// use log::{error, info, warn, LevelFilter, Record};
// use dotenvy::dotenv;
use env_logger::{Builder, Env};
// use log::{error, info, warn, trace, debug, Record, Level, Metadata, LevelFilter, SetLoggerError, set_boxed_logger, set_max_level};
use colored::*;
use log::{debug, error, info, trace, warn};
use std::{
    // collections::{HashMap, HashSet},
    env as stdenv,
    fs::File,
    io::{
        BufRead,
        BufReader,
        Error as IOError,
        ErrorKind,
        // self,
        // Result as IOResult,
        // Read,
        Write,
    },
    path::{
        Path,
        // PathBuf
    },
    // fmt::format!,
    process::{
        // self,
        exit,
        id as process_id,
        Command,
        // Stdio
    },
    string::String as IOString,
    // sync::Mutex,
    vec::Vec as IOVec,
};

use chrono::{
    offset::{
        TimeZone,
        // Offset,
    },
    DateTime,
    Datelike,
    Local,
    Duration,
    // FixedOffset,
    // Local,

    // Utc,
    NaiveDate,
    NaiveDateTime,
    // FixedOffset,
    // LocalResult,
    // TimeZone,
    Utc,
};

const VERSION: &str = stdenv!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = stdenv!("CARGO_PKG_DESCRIPTION");
const NAME: &str = stdenv!("CARGO_PKG_NAME");

async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

async fn post_customer(item: web::Json<Customer>) -> impl Responder {
    let creds = StripeClient::new().into();
    match item.0.async_post(creds).await {
        Ok(cust) => HttpResponse::Ok().json(cust),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

async fn post_charge(item: web::Json<Charge>) -> impl Responder {
    let creds = StripeClient::new().into();
    match item.0.async_post(creds).await {
        Ok(charge) => HttpResponse::Ok().json(charge),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let this_script_relative_path = stdenv::args().next().unwrap_or_default();
    let this_script_name = std::path::Path::new(&this_script_relative_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_owned();
    let this_script_absolute_pathbuf =
        std::env::current_exe().expect("Failed to get the current executable path");
    let this_script_absolute_path = std::path::Path::new(&this_script_absolute_pathbuf);

    // Initialize the logger
    setup_logger();

    // Load .env .env_cors files and log error if not found
    load_env_file();
    check_env_cors();

    dotenv().ok();

    info!(
        "\x1b[01;35m # THIS SCRIPT NAME\x1b[38;5;93m:\x1b[38;5;1m {}",
        this_script_name
    );
    info!(
        "\x1b[01;35m # THIS SCRIPT RELATIVE PATH\x1b[38;5;93m:\x1b[38;5;1m {}",
        this_script_relative_path
    );
    info!(
        "\x1b[01;35m # THIS SCRIPT ABSOLUTE PATH\x1b[38;5;93m:\x1b[38;5;1m {:?}",
        this_script_absolute_path
    );
    info!("PID: {}", std::process::id());

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

            let pwd = stdenv::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
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
    }
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
                    .arg(format!("lsof -i :{} -t -sTCP:LISTEN", target_port))
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
    }

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
            .configure(|cfg| {
                cfg.route("/health", web::get().to(health))
                    .route("/customer", web::post().to(post_customer))
                    .route("/charge", web::post().to(post_charge));
            })
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
    }
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

//     if let Err(e) = server.run().await {
//         eprintln!("💥 Server runtime failure: {e:?}");
//     }

//     Ok(())
// }
