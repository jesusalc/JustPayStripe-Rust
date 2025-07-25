use notify::{Watcher, RecursiveMode, recommended_watcher, EventKind};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::process::{Command, Stdio};
use std::env;
use std::path::Path;
use std::fs;
use std::io::{BufRead, BufReader, Write};
// use log::{info, warn, error};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use chrono::Local;
use env_logger::{Env, Builder};
use colored::*;
use log::{info, error};

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    println!("{} version:{} Usage: {}  --watch <path> --process <script> [--timeout <duration>] [--timeout-process <script>]", this_script_name, version, this_script_name);
    println!("Watches a folder and when a file appears it sends the name as argument to another process. ");
    println!("This one has timer optional which counts time after last file received and triggers choice timeout1 or timeout2");
    println!("\\___ choice timeout 1 is when no timeout-process is passed then it ends the server");
    println!("\\___ choice timeout 2 is when when timeout-process is passed then it triggers it, and keeps going");
    println!("Options:");
    println!("  --help, -h             Show this help message");
    println!("  --watch                Required. Path to the directory to watch");
    println!("  --process              Required. Script to execute when a file is found, with the file path as an argument");
    println!("  --timeout              Optional. Timeout duration for inactivity (e.g., '10ns', '10ms', '10s', '5m')");
    println!("  --timeout-process      Optional. Script to execute when timeout occurs");
}

fn init_logger() {
    let this_script_name = Path::new(&env::args().next().unwrap_or_default())
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_owned();
    let this_script_name_with_version = format!("{}_{}", this_script_name, VERSION);
// env_logger::Builder::from_env(Env::default().default_filter_or("info"))
//     .format(move |buf, record| {
//         writeln!(
//             buf,
//             "[{} {} {}] {}",
//             this_script_name,
//             Local::now().format("%Y-%m-%dT%H:%M:%S"),
//             record.level(),
//             record.args()
//         )
//     })
//     .init();
        // Builder::from_env(Env::default().default_filter_or("info"))
        // .format(move |buf, record| {
        //     let level = match record.level() {
        //         log::Level::Error => format!("{}", record.level()).red(),
        //         log::Level::Warn => format!("{}", record.level()).yellow(),
        //         log::Level::Info => format!("{}", record.level()).green(),
        //         log::Level::Debug => format!("{}", record.level()).blue(),
        //         log::Level::Trace => format!("{}", record.level()).purple(),
        //     };

        //     writeln!(
        //         buf,
        //         "[{} {} {}] {}",
        //         this_script_name_with_version.to_string().dimmed(),
        //         Local::now().format("%Y-%m-%dT%H:%M:%S").to_string().dimmed(),
        //         level,
        //         record.target(),
        //         record.args()
        //     )
        // })
        // .init();
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(move |buf, record| {
            let level = match record.level() {
                log::Level::Error => format!("{}", record.level()).red(),
                log::Level::Warn =>  format!(" {}", record.level()).yellow(),
                log::Level::Info =>  format!(" {}", record.level()).green(),
                log::Level::Debug => format!("{}", record.level()).blue(),
                log::Level::Trace => format!("{}", record.level()).purple(),
            };

            writeln!(
                buf,
                "[{} {}]{}: {}",
                this_script_name_with_version.to_string().dimmed(),
                Local::now().format("%Y%m%d %H:%M:%S").to_string().dimmed(),
                level,
                record.args()
            )
        })
        .init();
}

fn parse_duration(duration_str: &str) -> Result<Duration, &'static str> {
    let mut chars = duration_str.chars().peekable();
    let mut num_str = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_digit(10) {
            num_str.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    let num: u64 = num_str.parse().map_err(|_| "Invalid number in duration")?;
    let unit = chars.collect::<String>();
    match unit.as_str() {
        "ns" => Ok(Duration::from_nanos(num)),
        "ms" => Ok(Duration::from_millis(num)),
        "s" => Ok(Duration::from_secs(num)),
        "m" => Ok(Duration::from_secs(num * 60)),
        _ => Err("Invalid duration unit"),
    }
}

fn handle_event(event: notify::Result<notify::Event>, process_script: &str, retry_tx: Sender<String>) {
    if let Ok(event) = event {
        if let EventKind::Create(_) = event.kind {
            for path in event.paths {
                info!("New file detected: {:?}", path);
                if let Err(e) = start_process(process_script, &path) {
                    error!("Failed to start process: {:?}. Retrying...", e);
                    retry_tx.send(path.to_string_lossy().to_string()).expect("Failed to send to retry queue");
                }
            }
        }
    }
}

fn start_process(process_script: &str, path: &Path) -> Result<(), std::io::Error> {
    let mut child = Command::new(process_script)
        .arg(path.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    for line in stdout_reader.lines() {
        println!("stdout: {}", line.unwrap());
    }

    for line in stderr_reader.lines() {
        println!("stderr: {}", line.unwrap());
    }

    match child.wait() {
        Ok(status) => {
            if status.success() {
                info!("Process succeeded with status: {}", status);
            } else {
                error!("Process failed with status: {}", status);
            }
        },
        Err(e) => error!("Failed to wait on child process: {:?}", e),
    }

    info!("Process triggered: {:?}", process_script);
    Ok(())
}

fn retry_thread(retry_rx: Receiver<String>, process_script: String, retry_delay: Duration) {
    let mut retry_queue: VecDeque<String> = VecDeque::new();
    loop {
        while let Ok(path) = retry_rx.try_recv() {
            retry_queue.push_back(path);
        }

        if let Some(path) = retry_queue.pop_front() {
            thread::sleep(retry_delay);
            info!("Retrying file: {:?}", path);
            if let Err(e) = start_process(&process_script, Path::new(&path)) {
                error!("Retry failed: {:?}. Requeueing...", e);
                retry_queue.push_back(path);
            }
        } else {
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn timeout_thread(rx: Receiver<notify::Result<notify::Event>>, process_script: String, timeout_duration: Duration, timeout_process_script: Option<String>, retry_tx: Sender<String>) {
    let last_event = Arc::new(Mutex::new(Instant::now()));

    let last_event_clone = Arc::clone(&last_event);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let elapsed = Instant::now().duration_since(*last_event_clone.lock().unwrap());
            if elapsed >= timeout_duration {
                if let Some(script) = &timeout_process_script {
                    info!("Timeout reached, running timeout process: {}", script);
                    let mut child = Command::new(script)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to start timeout process");

                    let stdout = child.stdout.take().unwrap();
                    let stderr = child.stderr.take().unwrap();
                    let stdout_reader = BufReader::new(stdout);
                    let stderr_reader = BufReader::new(stderr);

                    for line in stdout_reader.lines() {
                        println!("stdout: {}", line.unwrap());
                    }

                    for line in stderr_reader.lines() {
                        println!("stderr: {}", line.unwrap());
                    }

                    match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                info!("Timeout process succeeded with status: {}", status);
                            } else {
                                error!("Timeout process failed with status: {}", status);
                            }
                        },
                        Err(e) => error!("Failed to wait on timeout process: {:?}", e),
                    }
                    *last_event_clone.lock().unwrap() = Instant::now();
                } else {
                    info!("Timeout reached, stopping observer");
                    std::process::exit(0);
                }
            }
        }
    });

    loop {
        match rx.recv() {
            Ok(event) => {
                *last_event.lock().unwrap() = Instant::now();
                handle_event(event, &process_script, retry_tx.clone());
            },
            Err(e) => error!("Watch error: {:?}", e),
        }
    }
}

fn main() {
    init_logger();
    println!("{:?}", std::process::id());
    info!("Starting application");

    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from("--help")) || args.contains(&String::from("-h")) {
        print_help();
        return;
    }

    let watch_index = args.iter().position(|x| x == "--watch");
    let process_index = args.iter().position(|x| x == "--process");
    let timeout_index = args.iter().position(|x| x == "--timeout");
    let timeout_process_index = args.iter().position(|x| x == "--timeout-process");

    if watch_index.is_none() || process_index.is_none() || args.len() <= watch_index.unwrap() + 1 || args.len() <= process_index.unwrap() + 1 {
        error!("Error: Missing required arguments");
        print_help();
        std::process::exit(1);
    }

    let watch_path = &args[watch_index.unwrap() + 1];
    let process_script = args[process_index.unwrap() + 1].clone();
    let timeout_duration = if let Some(index) = timeout_index {
        if args.len() <= index + 1 {
            error!("Error: Missing duration for --timeout");
            print_help();
            std::process::exit(1);
        }
        match parse_duration(&args[index + 1]) {
            Ok(duration) => Some(duration),
            Err(err) => {
                error!("Error: {}", err);
                print_help();
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let timeout_process_script = if let Some(index) = timeout_process_index {
        if args.len() <= index + 1 {
            error!("Error: Missing script for --timeout-process");
            print_help();
            std::process::exit(1);
        }
        Some(args[index + 1].clone())
    } else {
        None
    };

    if !Path::new(watch_path).is_dir() {
        error!("Error: --watch path is not a directory or cannot be accessed");
        std::process::exit(1);
    }

    if !Path::new(&process_script).is_file() || fs::metadata(&process_script).unwrap().permissions().readonly() {
        error!("Error: --process script is not a file or cannot be executed");
        std::process::exit(1);
    }

    if let Some(script) = &timeout_process_script {
        if !Path::new(script).is_file() || fs::metadata(script).unwrap().permissions().readonly() {
            error!("Error: --timeout-process script is not a file or cannot be executed");
            std::process::exit(1);
        }
    }

    info!("Watching directory: {}", watch_path);
    info!("Process to execute: {}", process_script);
    info!("PID: {}", std::process::id());

    let (tx, rx) = channel();
    let (retry_tx, retry_rx) = channel();
    let mut watcher = recommended_watcher(tx).unwrap();
    watcher.watch(Path::new(watch_path), RecursiveMode::Recursive).unwrap();

    let process_script_clone = process_script.clone();
    thread::spawn(move || {
        retry_thread(retry_rx, process_script_clone, Duration::from_secs(1));
    });

    if let Some(timeout) = timeout_duration {
        let process_script_clone = process_script.clone();
        let timeout_process_script_clone = timeout_process_script.clone();
        let retry_tx_clone = retry_tx.clone();
        let timeout_thread_handle = thread::spawn(move || {
            timeout_thread(rx, process_script_clone, timeout, timeout_process_script_clone, retry_tx_clone);
        });

        // Wait for the timeout thread to finish
        timeout_thread_handle.join().expect("Timeout thread panicked");
    } else {
        loop {
            match rx.recv() {
                Ok(event) => handle_event(event, &process_script, retry_tx.clone()),
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
    }

    info!("Exiting application");
}
