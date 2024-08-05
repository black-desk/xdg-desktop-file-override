use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::env;

pub fn get_config_file(config_file_name: &str) -> io::Result<(PathBuf, Box<dyn Read>)> {
    // Check XDG_CONFIG_HOME
    let xdg_config_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("HOME environment variable not set");
        format!("{}/.config", home)
    });
    let config_path = PathBuf::from(xdg_config_home).join(config_file_name);
    if config_path.exists() {
        return Ok((config_path.clone(), Box::new(File::open(config_path)?)));
    }

    // Check XDG_CONFIG_DIRS
    let xdg_config_dirs = env::var("XDG_CONFIG_DIRS").unwrap_or_else(|_| "/etc/xdg".to_string());
    let paths: Vec<PathBuf> = env::split_paths(&xdg_config_dirs).collect();
    for path in paths {
        let config_path = path.join(config_file_name);
        if config_path.exists() {
            return Ok((config_path.clone(), Box::new(File::open(config_path)?)));
        }
    }

    // Check default config path
    let default_config_path = PathBuf::from("/etc/xdg").join(config_file_name);
    if default_config_path.exists() {
        return Ok((default_config_path.clone(), Box::new(File::open(default_config_path)?)));
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Configuration file not found",
    ))
}

#[cfg(test)]
mod tests {
    use io::Write;

    use super::*;

    #[test]
    fn test_get_config_file_existing_file() {
        // Create a temporary directory
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");

        // Create a temporary config file
        let config_file_path = temp_dir.path().join("config.txt");

        let mut config_file = File::create(&config_file_path).expect("Failed to create config file");

        // Write some data to the config file
        config_file.write_all(b"Hello, World!").expect("Failed to write to config file");

        env::set_var("XDG_CONFIG_HOME", temp_dir.path().to_str().unwrap());
        // Call the get_config_file function
        let (path, mut reader) = get_config_file("config.txt").expect("Failed to get config file");

        // Assert that the returned path is correct
        assert_eq!(path, config_file_path);

        // Read the data from the reader and assert that it matches the written data
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer).expect("Failed to read from config file");
        assert_eq!(buffer, "Hello, World!");
    }

    #[test]
    fn test_get_config_file_nonexistent_file() {
        // Call the get_config_file function with a non-existent file
        let result = get_config_file("nonexistent.txt");

        // Assert that the function returns an error
        assert!(result.is_err());

        // Assert that the error kind is NotFound
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::NotFound);
    }
}