use clap::Command;
use log::{debug, info};
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Stdio};

#[derive(Debug, Deserialize)]
struct Config {
    version: String,
    generators: Vec<Generator>,
}

#[derive(Debug, Deserialize)]
struct Generator {
    filter: String,
    name: String,
    command: Vec<String>,
}

const CONFIG_FILE_PATH: &str = "xdg-desktop-file-override/config.yaml";

fn main() -> io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let command = Command::new("xdg-desktop-file-override")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Chen Linxuan <me@black-desk.cn>")
        .about("Override XDG desktop file according to configuration.")
        .long_about(include_str!("../docs/xdg-desktop-file-override.md"))
        .arg(
            clap::Arg::new("debug")
                .short('d')
                .long("debug")
                .help("Print debug information verbosely")
                .num_args(0)
                .required(false),
        )
        .subcommand(Command::new("clean").about("Remove all generated desktop files."))
        .subcommand(Command::new("generate").about("Generate override desktop files."));

    debug!("version: {}", command.get_version().unwrap());

    let matches = command.get_matches();

    if let Some(_matches) = matches.subcommand_matches("clean") {
        clean_generated_files()?;
        return Ok(());
    }

    if let Some(_matches) = matches.subcommand_matches("generate") {
        clean_generated_files()?;
        generate_files()?;
        return Ok(());
    }

    return Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Invalid subcommand",
    ));
}

fn generate_files() -> io::Result<()> {
    let config_path = get_config_path()?;
    debug!("Using configuration file at: {:?}", config_path);

    let config: Config = serde_yaml::from_reader(File::open(config_path)?)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // Print the version from the configuration file
    debug!("Config version: {}", config.version);

    // Get desktop files from XDG_DATA_DIRS
    let desktop_files = get_desktop_files()?;

    // Process each desktop file
    for desktop_file in desktop_files {
        let content = std::fs::read_to_string(&desktop_file)?;
        let mut new_content = content.clone();
        let mut updated = false;

        for generator in &config.generators {
            let re = Regex::new(&generator.filter).unwrap();
            if !re.is_match(desktop_file.file_name().unwrap().to_str().unwrap()) {
                continue;
            }

            // Print the name of the generator
            debug!(
                "Applying generator {} on {}",
                generator.name,
                desktop_file.display(),
            );

            let output = apply_generator(&generator.command, &new_content)?;
            if !output.status.success() {
                continue;
            }

            let generated_content = String::from_utf8_lossy(&output.stdout).to_string();
            if generated_content != new_content {
                new_content = generated_content;
                updated = true;
            }
        }

        if !updated {
            continue;
        }

        // Write new content to XDG_DATA_HOME/applications
        write_new_desktop_file(&desktop_file, &new_content)?;
    }

    Ok(())
}

fn get_config_path() -> io::Result<PathBuf> {
    // Check XDG_CONFIG_HOME
    let xdg_config_home = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("HOME environment variable not set");
        format!("{}/.config", home)
    });
    let config_path = PathBuf::from(xdg_config_home).join(CONFIG_FILE_PATH);
    if config_path.exists() {
        return Ok(config_path);
    }

    // Check XDG_CONFIG_DIRS
    let xdg_config_dirs = env::var("XDG_CONFIG_DIRS").unwrap_or_else(|_| "/etc/xdg".to_string());
    let paths: Vec<PathBuf> = env::split_paths(&xdg_config_dirs).collect();
    for path in paths {
        let config_path = path.join(CONFIG_FILE_PATH);
        if config_path.exists() {
            return Ok(config_path);
        }
    }

    // Check default config path
    let default_config_path = PathBuf::from("/etc/xdg").join(CONFIG_FILE_PATH);
    if default_config_path.exists() {
        return Ok(default_config_path);
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Configuration file not found",
    ))
}

fn get_desktop_files() -> io::Result<Vec<PathBuf>> {
    let xdg_data_dirs =
        env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
    let paths: Vec<PathBuf> = env::split_paths(&xdg_data_dirs).collect();
    let mut desktop_files = Vec::new();

    for path in paths {
        let applications_path = path.join("applications");
        if !applications_path.exists() {
            continue;
        }

        for entry in std::fs::read_dir(applications_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("desktop") {
                desktop_files.push(entry.path());
            }
        }
    }

    Ok(desktop_files)
}

fn apply_generator(command: &[String], input: &str) -> io::Result<std::process::Output> {
    let mut cmd = ProcessCommand::new(&command[0]);
    cmd.args(&command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to open stdin"))?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    Ok(output)
}

fn write_new_desktop_file(original_path: &PathBuf, content: &str) -> io::Result<()> {
    let xdg_data_home = env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| format!("{}/.local/share", env::var("HOME").unwrap()));
    let new_path = PathBuf::from(xdg_data_home)
        .join("applications")
        .join(original_path.file_name().unwrap());

    let mut final_content = content.to_string();
    let override_property = "X-XDG-Desktop-File-Override-Version";
    if !final_content.contains(override_property) {
        // Insert the property into the beginning of the main section
        let main_section_start =
            final_content.find("[Desktop Entry]").unwrap_or_else(|| 0) + "[Desktop Entry]".len();
        final_content.insert_str(
            main_section_start,
            &format!("\n{}={}", override_property, env!("CARGO_PKG_VERSION")),
        );
    }

    if new_path.exists() {
        return Ok(());
    }

    info!(
        "Writing new desktop file {:?}",
        original_path.file_name().unwrap()
    );

    return std::fs::write(new_path, final_content);
}

fn clean_generated_files() -> io::Result<()> {
    let xdg_data_home = env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| format!("{}/.local/share", env::var("HOME").unwrap()));
    let applications_path = PathBuf::from(xdg_data_home).join("applications");

    for entry in std::fs::read_dir(applications_path)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("desktop") {
            let content = std::fs::read_to_string(&entry.path())?;
            if content.contains("X-XDG-Desktop-File-Override-Version") {
                std::fs::remove_file(entry.path())?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_get_config_path() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("xdg-desktop-file-override/config.yaml");
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        File::create(&config_path).unwrap();

        env::set_var("XDG_CONFIG_HOME", dir.path().to_str().unwrap());
        let result = get_config_path().unwrap();
        assert_eq!(result, config_path);
    }

    #[test]
    fn test_get_desktop_files() {
        let dir = tempdir().unwrap();
        let applications_path = dir.path().join("applications");
        fs::create_dir_all(&applications_path).unwrap();
        File::create(applications_path.join("test.desktop")).unwrap();

        env::set_var("XDG_DATA_DIRS", dir.path().to_str().unwrap());
        let result = get_desktop_files().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file_name().unwrap(), "test.desktop");
    }

    #[test]
    fn test_apply_generator() {
        let command = vec![
            "sed".to_string(),
            "-e".to_string(),
            "s/foo/bar/".to_string(),
        ];
        let input = "foo";
        let output = apply_generator(&command, input).unwrap();
        let result = String::from_utf8_lossy(&output.stdout);
        assert_eq!(result, "bar");
    }

    #[test]
    fn test_write_new_desktop_file() {
        let dir = tempdir().unwrap();
        let applications_path = dir.path().join("applications");
        fs::create_dir_all(&applications_path).unwrap();
        let original_path = applications_path.join("test.desktop");
        let content = "[Desktop Entry]\nName=Test";

        env::set_var("XDG_DATA_HOME", dir.path().to_str().unwrap());
        write_new_desktop_file(&original_path, content).unwrap();

        let new_path = applications_path.join("test.desktop");
        let result = fs::read_to_string(new_path).unwrap();
        assert!(result.contains("X-XDG-Desktop-File-Override-Version=0.1.0"));
    }
}
