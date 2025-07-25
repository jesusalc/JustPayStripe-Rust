use super::*;

// fn load_cors_origins(path: &str) -> io::Result<Vec<String>> {
//     let file = File::open(path)?;
//     let buf = io::BufReader::new(file);
//     buf.lines().collect()
// }
pub fn check_env_cors() {
    let current_dir = stdenv::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env_cors_path = current_dir.join(".env_cors");

    if env_cors_path.exists() {
        info!(".env_cors file found at: {}", env_cors_path.display());
    } else {
        error!(
            ".env_cors file not found. Expected it at: {}",
            env_cors_path.display()
        );
    }
}

pub fn load_and_validate_cors_origins(path: &str) -> actix_web::Result<IOVec<IOString>, IOError> {
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
