use super::*;

pub fn load_env_file() {
    // Get the current directory
    let current_dir = stdenv::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
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
        info!(".env loading at: {}", env_path.display());
    }
}

pub fn load_env_var(key: &str, default: &str) -> String {
    stdenv::var(key).unwrap_or_else(|_| {
        if default == "/home/zeus" {
            stdenv::var("HOME").unwrap_or_else(|_| "/home".to_string())
        } else {
            default.to_string()
        }
    })
}
