# vestaboard-local

[![Crates.io](https://img.shields.io/crates/v/vestaboard_local.svg)](https://crates.io/crates/vestaboard_local)
[![Docs.rs](https://docs.rs/vestaboard_local/badge.svg)](https://docs.rs/vestaboard_local)
[![Build Status](https://github.com/nfang/vestaboard-local/actions/workflows/rust.yml/badge.svg)](https://github.com/nfang/vestaboard-local/actions/workflows/rust.yml)
[![License](https://img.shields.io/crates/l/vestaboard_local.svg)](https://crates.io/crates/vestaboard_local)

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


This is a template README for open-source Rust projects. It provides a structured outline to help you present your project effectively.

## Features


## Getting Started

### Prerequisites

To run this project, these environment variables need to be added to a `.env` file:
```
LOCAL_API_KEY
IP_ADDRESS
```
Optional environment variables for widgets.
```
WEATHER_API_KEY
```

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
| '/' | Slash | '?' | Question Mark | '°' | Degree |
| 'R' | Red | 'O' | Orange | 'Y' | Yellow |
| 'G' | Green | 'B' | Blue | 'V' | Violet |
| 'W' | White | 'K' | Black | 'b' | B |


## Examples
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

## Contributing


## Widgets

### Weather
https://www.weatherapi.com/api-explorer.aspx#forecast

## License


## Contact


## Acknowledgements

