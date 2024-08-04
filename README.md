# xdg-desktop-file-override

This is a Rust command-line application that generates new desktop files
to `XDG_DATA_HOME/applications` according to a configuration file.

New .desktop files generated by this program
will override the old ones provided by the system package manager.

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
      # I do not want my application to use DBusActivatable feature.
      '-e', 's/DBusActivatable=true/d',
    ]
  - name: fix-zeditor
    # Fix some issue in zeditor desktop file provided by upstream.
    filter: ^zed\\.desktop$
    command: [ 'sed',
      # Add missing StartupWMClass
      '-e', '/\\[Desktop Entry\\]/a StartupWMClass=dev.zed.Zed',
      # Remove StartupNotify
      '-e', '/StartupNotify=true/d',
    ]
```

The configuration file should be placed at
`~/.config/xdg-desktop-file-override/config.yaml`.

For more information about this configuration file and how this program work,
please check `xdg-desktop-file-override --help`.

NOTE: This program respects `$XDG_CONFIG_HOME` and `$XDG_CONFIG_DIRS`.

## Installation

You need to have Rust installed on your machine.
Then, you can clone this repository and build the project:

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
