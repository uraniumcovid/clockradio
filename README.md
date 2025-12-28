# Clock Radio TUI

A simple terminal-based clock radio application written in Rust with a beautiful pink and black color scheme.

### Building and Running

Option 1: Use the provided script (recommended for NixOS):
```bash
./run.sh
```

Option 2: Build manually with Cargo:
```bash
cargo build --release
cargo run --release
```

Option 3: Install and run from anywhere:
```bash
cargo install --path .
clockradio
```

### Controls

- **q**: Quit application
- **a**: Set alarm (enter time in HH:MM format)
- **Enter**: Confirm alarm time
- **Esc**: Cancel alarm dialog
- **Backspace**: Edit alarm input

### Weather Setup

To enable weather information:

1. Get a free API key from [OpenWeatherMap](https://openweathermap.org/api)
2. Set the environment variable:
   ```bash
   export OPENWEATHER_API_KEY="your_api_key_here"
   ```
3. Run the application

The weather location is currently set to London. You can modify the location in the source code (`src/main.rs:75`).

## Requirements

- Rust 1.70 or later
- Terminal with color support
- Internet connection for weather data (optional)

## Dependencies

- `ratatui`: Terminal UI framework
- `crossterm`: Cross-platform terminal manipulation
- `chrono`: Date and time handling
- `tokio`: Async runtime
- `reqwest`: HTTP client for weather API
- `serde`: JSON serialization

## Color Scheme

The application uses a pink and black color scheme:
- Background: Black
- Primary text/borders: Deep Pink (RGB: 255, 20, 147)
- Secondary text: Light Pink (RGB: 255, 182, 193)
