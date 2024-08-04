# xdg-desktop-file-override

This is a Rust command-line application that generates new desktop files to `XDG_DATA_HOME/applications` according to a configuration file. The new .desktop files will override the old ones provided by the system or packager.

## Features

- Customize the command used to run some desktop applications
- Fix applications with wrong `StartupWMClass/StartupNotify` property
- Add new `MimeType` to open some file type with the application

## Usage

```bash
xdg-desktop-file-override [FLAGS] [SUBCOMMAND]
```

### Flags

- `-h, --help`: Prints help information
- `-V, --version`: Prints version information

### Subcommands

- `clean`: Remove the generated desktop files

## Configuration

The configuration file is a YAML file that looks like this:

```yaml
version: 0.1.0
generators:
  - name: remove-all-dbusactivatable-equals-true
    filter: .*
    command: [ 'sed',
      '-e', 's/DBusActivatable=true/d',
    ]
  - filter: ^zed\\.desktop$
    name: fix-zeditor
    command: [ 'sed',
      # Add missing StartupWMClass
      '-e', '/\\[Desktop Entry\\]/a StartupWMClass=dev.zed.Zed',
      # Remove StartupNotify
      '-e', '/StartupNotify=true/d',
    ]
```

The configuration file should be placed at `XDG_CONFIG_DIRS/xdg-desktop-file-override/config.yaml`.

## Installation

You need to have Rust installed on your machine. Then, you can clone this repository and build the project:

```bash
git clone https://github.com/username/xdg-desktop-file-override.git
cd xdg-desktop-file-override
cargo build --release
```

The executable will be located in the `target/release` directory.

## Tests

You can run the tests with:

```bash
cargo test
```
