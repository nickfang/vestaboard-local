# vestaboard-local

<!-- [![Crates.io](https://img.shields.io/crates/v/vestaboard_local.svg)](https://crates.io/crates/vestaboard_local)
[![Docs.rs](https://docs.rs/vestaboard_local/badge.svg)](https://docs.rs/vestaboard_local)
[![Build Status](https://github.com/nfang/vestaboard-local/actions/workflows/rust.yml/badge.svg)](https://github.com/nfang/vestaboard-local/actions/workflows/rust.yml)
[![License](https://img.shields.io/crates/l/vestaboard_local.svg)](https://crates.io/crates/vestaboard_local) -->

This project allows a user to connect to their vesta board locally

## Table of Contents

- [Features](#features)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Examples](#examples)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)
- [Acknowledgements](#acknowledgements)
- [Disclaimer (if applicable)](#disclaimer-if-applicable)


## Features


## Getting Started

### Prerequisites

Enable Local API on your Vestaboard: https://docs-v1.vestaboard.com/local

Add these environment variables a `.env` file:
- `LOCAL_API_KEY` - Key received after enabling Local API<br>
- `IP_ADDRESS` - Local IP address of the Vestaboard

Optional environment variables for widgets:
- `WEATHER_API_KEY` - https://www.weatherapi.com/docs/ (Getting Started)

### Installation
1. Clone the repository:
    ```sh
    git clone https://github.com/nfang/vestaboard-local.git
    cd vestaboard-local
    ```

2. Build the project:
    ```sh
    cargo build
    ```

3. Run the project:
    ```sh
    cargo run
    ```
    or run it from the target folder
    ```
    ./target/debug/vbl
    ```

## Configuration

The application uses a configuration file located at `data/vblconfig.toml`. This file is automatically created with default values when you first run the application.

### Configuration Options

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `log_level` | String | `"info"` | Controls the verbosity of file logging. Options: `"off"`, `"error"`, `"warn"`, `"info"`, `"debug"`, `"trace"` |
| `log_file_path` | String | `"data/vestaboard.log"` | Path to the log file (relative to application directory) |
| `console_log_level` | String (optional) | Same as `log_level` | Controls console output verbosity. If not specified, uses `log_level` |
| `schedule_file_path` | String | `"data/schedule.json"` | Path to the schedule file for storing scheduled tasks |
| `schedule_backup_path` | String | `"data/schedule_backup.json"` | Path to the schedule backup file |

### Example Configuration

```toml
# Vestaboard Local Configuration File
# This file controls logging and file paths for the vestaboard-local application

# Log level for file logging
# Options: "off", "error", "warn", "info", "debug", "trace"
# Default: "info"
log_level = "debug"

# Path to the log file (relative to the application directory)
# Default: "data/vestaboard.log"
log_file_path = "data/vestaboard.log"

# Log level for console output (optional)
# If not specified, uses the same level as log_level
# Options: "off", "error", "warn", "info", "debug", "trace"
console_log_level = "info"

# Schedule file paths
# These control where schedule data is stored and backed up
# Default: "data/schedule.json" and "data/schedule_backup.json"
schedule_file_path = "data/schedule.json"
schedule_backup_path = "data/schedule_backup.json"
```

### Configuration Notes

- **Backward Compatibility**: If you have an existing configuration file missing the newer options (like schedule paths), the application will use the default values.
- **Relative Paths**: All file paths are relative to the application's working directory.
- **Automatic Creation**: If no configuration file exists, the application creates one with default values on first run.

### Usage

Messages can be passed in as a text file or a string. Only characters below are allowed.

| Character | Description | Character | Description | Character | Description |
| :-: | - | :-: | - | :-: | - |
| ' ' | Blank | 'a' | A | 'b' | B |
| 'c' | C | 'd' | D | 'e' | E |
| 'f' | F | 'g' | G | 'h' | H |
| 'i' | I | 'j' | J | 'k' | K |
| 'l' | L | 'm' | M | 'n' | N |
| 'o' | O | 'p' | P | 'q' | Q |
| 'r' | R | 's' | S | 't' | T |
| 'u' | U | 'v' | V | 'w' | W |
| 'x' | X | 'y' | Y | 'z' | Z |
| '1' | 1 | '2' | 2 | '3' | 3 |
| '4' | 4 | '5' | 5 | '6' | 6 |
| '7' | 7 | '8' | 8 | '9' | 9 |
| '0' | 0 | '!' | Exclamation Mark | '@' | At |
| '#' | Pound | '$' | Dollar | '(' | Left Parenthesis |
| ')' | Right Parenthesis | '-' | Hyphen | '+' | Plus |
| '&' | Ampersand | '=' | Equal | ';' | Semicolon |
| ':' | Colon | ''' | Single Quote | '"' | Double Quote |
| '%' | Percent | ',' | Comma | '.' | Period |
| '/' | Slash | '?' | Question Mark | 'D' | Degree |
| 'R' | Red | 'O' | Orange | 'Y' | Yellow |
| 'G' | Green | 'B' | Blue | 'V' | Violet |
| 'W' | White | 'K' | Black | 'b' | B |


## Examples
See possible commands.
```
vbl
```
To send a center-aligned string to the Vestaboard:
```
vbl send text "hello from vestaboard local."
```
Sending a message to the Vestaboard in the arrangement that is in the text file.
```
vbl send file ./text.txt
```
Using the weather widget.
```
vbl send weather
```
Preview message.
```
cargo run -- send -d sat-word
```

## Contributing

## Widgets

### `weather` - Current weather

Pulls data from:
https://www.weatherapi.com/api-explorer.aspx#forecast

API Error Codes
If there is an error, API response contains error message including error code for following 4xx HTTP Status codes.

| HTTP Status Code | Error code | Description |
|-----------------|------------|-------------|
| 401 | 1002 | API key not provided. |
| 400 | 1003 | Parameter 'q' not provided. |
| 400 | 1005 | API request url is invalid |
| 400 | 1006 | No location found matching parameter 'q' |
| 401 | 2006 | API key provided is invalid |
| 403 | 2007 | API key has exceeded calls per month quota. |
| 403 | 2008 | API key has been disabled. |
| 403 | 2009 | API key does not have access to the resource. Please check pricing page for what is allowed in your API subscription plan. |
| 400 | 9000 | Json body passed in bulk request is invalid. Please make sure it is valid json with utf-8 encoding. |
| 400 | 9001 | Json body contains too many locations for bulk request. Please keep it below 50 in a single request. |
| 400 | 9999 | Internal application error. |


### `sat-word` - Random SAT word and definition
Word bank located at: `src/widgets/sat_words/words.txt`

## License

Copyright (c) 2024 Nicholas Fang

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/nickfang/vestaboard-local/blob/main/LICENSE)

## Contact


## Acknowledgements

