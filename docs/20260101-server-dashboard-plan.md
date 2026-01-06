# Vestaboard Local Server & Physical Button Plan

## Goals

1. **REST Server** - Receive HTTP calls to trigger Vestaboard commands
2. **Web Interface** - Simple HTML page with buttons to send commands
3. **Physical Button** - Raspberry Pi button that triggers commands via the server

All local to the network - no cloud required.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Local Network                           │
│                                                             │
│  ┌──────────────┐      HTTP/REST      ┌─────────────────┐  │
│  │  Web Page    │ ──────────────────► │                 │  │
│  │  (Browser)   │                     │  vbl server     │  │
│  └──────────────┘                     │  (Rust/Axum)    │  │
│                                       │                 │  │
│  ┌──────────────┐      HTTP/REST      │    ┌───────┐    │  │
│  │ Raspberry Pi │ ──────────────────► │    │Widgets│    │  │
│  │ + Button     │                     │    └───────┘    │  │
│  └──────────────┘                     │        │        │  │
│                                       │        ▼        │  │
│                                       │  ┌──────────┐   │  │
│                                       │  │Vestaboard│   │  │
│                                       │  │ (device) │   │  │
│                                       └──┴──────────┴───┘  │
└─────────────────────────────────────────────────────────────┘
```

## Current CLI Commands

| Command | Description |
|---------|-------------|
| `vbl show text <MSG>` | Display custom text |
| `vbl show weather` | Show weather for Austin, TX |
| `vbl show sat-word` | Random SAT vocabulary word |
| `vbl show jokes` | Display a joke |
| `vbl show clear` | Clear the board |
| `vbl show file <PATH>` | Display text from file |
| `vbl schedule add/list/remove/clear/preview` | Manage scheduled tasks |
| `vbl cycle [repeat]` | Run through scheduled tasks |
| `vbl daemon` | Background process for scheduled tasks |

## Proposed Server Implementation

### New Command

```
vbl server [--port 3000]
```

### REST Endpoints

```
POST /api/show/text       {"message": "Hello World"}
POST /api/show/weather
POST /api/show/sat-word
POST /api/show/jokes
POST /api/show/clear
GET  /api/status          Health check
GET  /                    Serve HTML button interface
```

### Files to Add/Modify

| File | Change |
|------|--------|
| `Cargo.toml` | Add `axum`, `tower-http` dependencies |
| `src/server.rs` | New - HTTP routes calling existing widget resolver |
| `src/cli_setup.rs` | Add `Server` command variant |
| `src/main.rs` | Handle `Server` command |
| `src/web/index.html` | New - Simple HTML with buttons (embedded in binary) |

### Dependencies to Add

```toml
axum = "0.7"
tower-http = { version = "0.5", features = ["cors", "fs"] }
```

## Raspberry Pi Button Setup

### Hardware
- Any Pi with GPIO (Pi Zero W is sufficient)
- Momentary button between GPIO pin and ground
- Use internal pull-up resistor

### Software (Python)

```python
import RPi.GPIO as GPIO
import requests
import time

BUTTON_PIN = 17
SERVER_URL = "http://vestaboard-server.local:3000"

GPIO.setmode(GPIO.BCM)
GPIO.setup(BUTTON_PIN, GPIO.IN, pull_up_down=GPIO.PUD_UP)

def on_button_press(channel):
    requests.post(f"{SERVER_URL}/api/show/weather")

GPIO.add_event_detect(BUTTON_PIN, GPIO.FALLING,
                      callback=on_button_press,
                      bouncetime=300)

while True:
    time.sleep(1)
```

Run as systemd service for auto-start on boot.

## Implementation Steps

1. Add Axum server with REST endpoints for each widget
2. Create simple HTML page with buttons (one per command)
3. Embed HTML in Rust binary using `include_str!()`
4. Test locally with curl/browser
5. Set up Pi with button and Python script
6. Configure server to run as systemd service on home server

## Configuration

Existing `.env` credentials will be shared:
- `LOCAL_API_KEY` - Vestaboard API key
- `IP_ADDRESS` - Vestaboard device IP
- `WEATHER_API_KEY` - weatherapi.com key

Server-specific config could be added to `data/vblconfig.toml`:
```toml
server_port = 3000
server_bind_address = "0.0.0.0"
```

## Notes

- Server reuses all existing widget logic - no duplication
- Pi button calls server (centralized logic, easier updates)
- Fallback: Pi could run CLI directly if server is down
