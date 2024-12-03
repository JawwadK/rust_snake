# 🐍 Rust Snake Game

[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](https://www.rust-lang.org)
[![GGEZ](https://img.shields.io/badge/GGEZ-0.9-blue.svg)](https://ggez.rs/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](https://opensource.org/licenses/MIT)

A modern implementation of Snake with particle effects, multiple difficulties, and persistent high scores - built with Rust and GGEZ.

## ✨ Features

- 🎮 Four difficulty levels with unique speed/score multipliers
- 📊 Persistent high scores per difficulty
- 🎯 Particle effects and smooth animations
- 🔊 Sound effects for actions
- ⚡ Fast and efficient Rust implementation

## 🚀 Quick Start

```bash
# Clone and enter directory
git clone [your-repo-url]
cd snake_game

# Create required files
echo "[]" > high_scores.json
mkdir resources

# Add sound files to resources/
# - eat.wav
# - game_over.wav

# Run the game
cargo run
```

## 🎮 Controls

- **↑←↓→**: Move snake
- **ESC**: Pause/Menu
- **R**: Restart
- **Enter**: Select menu items

## 🛠️ Built With

```toml
[dependencies]
ggez = "0.9"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

## 📝 License

This project is [MIT](LICENSE) licensed.
======