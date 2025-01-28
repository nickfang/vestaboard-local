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
vbl text -m "hello from vestaboard local."
```
Sending a message to the Vestaboard in the arrangement that is in the text file.
```
vbl text -f ./text.txt
```
Using the weather widget.
```
vbl weather
```
Preview message.
```
cargo run -- -t sat-word
```

## Contributing


## Widgets

### `weather` - Current weather

Pulls data from:
https://www.weatherapi.com/api-explorer.aspx#forecast

### `sat-word` - Random SAT word and definition
Word bank located at: `src/widgets/sat_words/words.txt`

## License

Copyright (c) 2024 Nicholas Fang

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/nickfang/vestaboard-local/blob/main/LICENSE)

## Contact


## Acknowledgements

