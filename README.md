# Deterministic P2P Pong

A cross-platform, deterministic multiplayer Pong game built in Rust with a shared core library, terminal client, and planned web client featuring direct P2P networking via WebRTC.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Rust Version](https://img.shields.io/badge/rust-1.75+-orange)

## 🎮 Quick Start

```bash
# Clone the repository
git clone https://github.com/asyed94/project-pong.git
cd project-pong

# Run the terminal client
cargo run --bin terminal-client

# Or run the CLI testing harness
cargo run --bin cli_harness
```

## 📋 Table of Contents

- [Features](#-features)
- [Installation](#-installation)
- [Usage](#-usage)
- [Development](#-development)
- [Architecture](#-architecture)
- [Data Model](#-data-model)
- [API Documentation](#-api-documentation)
- [Contributing](#-contributing)
- [License](#-license)

## ✨ Features

- **Deterministic Physics**: Fixed-point arithmetic ensures identical gameplay across all platforms
- **Cross-Platform**: Shared Rust core compiles to native and WebAssembly
- **[TODO] Lockstep Networking**: Synchronized gameplay for lag-free multiplayer experience
- **[TODO] Direct P2P**: WebRTC DataChannel with manual SDP exchange (no servers required)
- **Multiple Clients**: Terminal UI and fully functional web interface with mobile support
- **Local Modes**: Play against AI, wall, or local second player
- **Serializable State**: Complete game state snapshots for synchronization

## 🚀 Installation

### Prerequisites

- **Rust**: Version 1.75 or later
- **Development Environment**: Optional but recommended: [devbox](https://www.jetpack.io/devbox)

### Using Devbox (Recommended)

```bash
# Install devbox if you haven't already
curl -fsSL https://get.jetpack.io/devbox | bash

# Enter the development environment
devbox shell

# Build the project
cargo build
```

### Manual Installation

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/asyed94/project-pong.git
cd project-pong
cargo build --release
```

### Building for WebAssembly

```bash
# Install wasm-pack
cargo install wasm-pack

# Build the WASM module
cd pong_core
wasm-pack build --target web --out-dir ../clients/web/wasm
```

## 🎯 Usage

### Terminal Client

The terminal client provides a full TUI experience using ratatui:

```bash
# Run the terminal client
cargo run --bin terminal-client

# [TODO] With specific configuration (CLI args not implemented)
# cargo run --bin terminal-client -- --tick-rate 60 --max-score 11
```

**Controls:**

- `↑/↓` or `W/S`: Move paddles (Player 1: W/S, Player 2: Arrow keys)
- `Space`: Ready/Start game
- `ESC`: Back to main menu
- `Q`: Quit

**Game Modes:**

- **[TODO] Host**: Create a game and share your SDP offer
- **[TODO] Join**: Join a game using the host's SDP offer
- **Local**: Local gameplay with AI, wall, or second player modes
- **Quit**: Exit the application

### CLI Harness

For testing and development, use the CLI harness to run headless simulations:

```bash
# Run basic simulation (interactive two-player game)
cargo run --bin cli_harness

# [TODO] Run with custom parameters (CLI args not implemented)
# cargo run --bin cli_harness -- --ticks 1000 --left-ai --right-ai

# [TODO] Test deterministic behavior (CLI args not implemented)
# cargo run --bin cli_harness -- --seed 12345 --verify-determinism
```

### Web Client

The web client provides a fully functional game experience with mobile-friendly controls and DOM-based rendering:

```bash
# Build WASM module
cd pong_core
wasm-pack build --target web --out-dir ../clients/web/wasm

# Install dependencies and run development server
cd ../clients/web
npm install
npm run dev

# Open http://localhost:5173 (or the URL shown by Vite)
```

**Web Features:**

- **Mobile Support**: Touch-friendly controls with drag gestures
- **Local Game Modes**: Play against AI, wall, or local second player
- **WASM Integration**: Rust core runs natively in the browser
- **Responsive Design**: Works on desktop and mobile devices
- **Real-time Rendering**: 60fps game loop with smooth animations

**Controls:**

- **Desktop**: Click and drag to move paddle, spacebar for ready/start
- **Mobile**: Touch and drag the paddle area, tap ready button
- **Game Modes**: Select AI, Wall, or Local multiplayer from the menu

## 🛠 Development

### Project Structure

```
repo/
├── pong_core/              # Shared game engine (Rust lib)
│   ├── src/
│   │   ├── lib.rs         # Public API exports
│   │   ├── types.rs       # Core types and fixed-point math
│   │   ├── game.rs        # Game state and logic
│   │   ├── physics.rs     # Physics simulation
│   │   ├── serialization.rs # State serialization
│   │   └── wasm.rs        # WebAssembly bindings
│   └── Cargo.toml
├── cli_harness/           # Testing harness
│   ├── src/main.rs
│   └── Cargo.toml
├── clients/
│   ├── terminal/          # Terminal UI client
│   │   ├── src/
│   │   │   ├── main.rs    # Application entry point
│   │   │   ├── app.rs     # Application state
│   │   │   ├── ui.rs      # TUI rendering
│   │   │   └── event.rs   # Input handling
│   │   └── Cargo.toml
│   └── web/               # Web client (TypeScript + WASM)
│       ├── index.html     # Main HTML entry point
│       ├── package.json   # Node.js dependencies
│       ├── vite.config.js # Vite build configuration
│       ├── tsconfig.json  # TypeScript configuration
│       ├── wasm/          # Generated WASM files
│       └── src/
│           ├── main.ts    # Application entry point
│           ├── game.ts    # Game loop and state
│           ├── GameRenderer.ts    # Rendering engine
│           ├── GameStateManager.ts # State management
│           ├── InputManager.ts    # Input handling
│           ├── MobileController.ts # Mobile controls
│           ├── screens.ts # Screen navigation
│           ├── ai.ts      # AI opponent logic
│           ├── types.ts   # TypeScript type definitions
│           └── styles/
│               └── main.css # Styling
├── Cargo.toml             # Workspace configuration
└── design-spec.md         # Detailed technical specification
```

### Building Different Targets

```bash
# Native debug build
cargo build

# Native release build
cargo build --release

# WASM build for web
cd pong_core
wasm-pack build --target web --features wasm

# Build all workspace members
cargo build --workspace

# Build specific binary
cargo build --bin terminal-client
cargo build --bin cli_harness
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p pong_core

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_deterministic_simulation

# Test WASM compatibility
cd pong_core
wasm-pack test --headless --firefox --features wasm
```

### Development Workflow

1. **Core Changes**: Modify `pong_core/` for game logic
2. **Test**: Run `cargo test -p pong_core` to verify changes
3. **Terminal Client**: Test with `cargo run --bin terminal-client`
4. **CLI Testing**: Use `cargo run --bin cli_harness` for automated testing
5. **WASM Build**: Rebuild WASM if core changes affect web client

### Code Style

- **Rust 2021 Edition**: Modern Rust features enabled
- **Fixed-Point Math**: All physics calculations use `Fx` type (16.16 format)
- **Deterministic**: No floating-point operations in simulation
- **Error Handling**: Use `Result` types for fallible operations
- **Documentation**: Document all public APIs with `///` comments

## 🏗 Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    DETERMINISTIC PONG ARCHITECTURE              │
└─────────────────────────────────────────────────────────────────┘

         ┌─────────────────┐                    ┌─────────────────┐
         │  Terminal Client│                    │   Web Client    │
         │   (ratatui)     │                    │  (JS + WASM)    │
         └─────────────────┘                    └─────────────────┘
                   │                                      │
                   │             Network Layer            │
                   │        (WebRTC DataChannel)          │
                   │                                      │
         ┌─────────┴─────────┐                  ┌─────────┴─────────┐
         │   Lockstep Net    │ ◄──────────────► │   Lockstep Net    │
         │   (tick sync)     │   Input/Snapshot │   (tick sync)     │
         └─────────┬─────────┘                  └─────────┬─────────┘
                   │                                      │
         ┌─────────┴─────────┐                  ┌─────────┴─────────┐
         │    pong_core      │                  │    pong_core      │
         │    (native)       │                  │     (WASM)        │
         └───────────────────┘                  └───────────────────┘

                            ┌─────────────────────┐
                            │     SHARED CORE     │
                            │                     │
                            │  ┌───────────────┐  │
                            │  │  Game State   │  │
                            │  │   (tick N)    │  │
                            │  └───────────────┘  │
                            │  ┌───────────────┐  │
                            │  │   Physics     │  │
                            │  │ (fixed-point) │  │
                            │  └───────────────┘  │
                            │  ┌───────────────┐  │
                            │  │ Serialization │  │
                            │  │  (snapshots)  │  │
                            │  └───────────────┘  │
                            └─────────────────────┘
```

### Component Relationships

#### pong_core (Shared Library)

- **Deterministic Engine**: Fixed-point physics simulation
- **Cross-Platform**: Compiles to native Rust and WebAssembly
- **Stateless API**: Pure functions for stepping simulation
- **Serializable**: Complete state snapshots for networking

#### Lockstep Networking

- **Tick Synchronization**: Both clients must have inputs for tick N before advancing
- **Input Broadcasting**: Local inputs sent to remote peer each tick
- **State Synchronization**: Periodic snapshots for resync if needed
- **Fault Tolerance**: Handle missing/late packets gracefully

#### Client Implementations

- **Terminal**: Native Rust using ratatui for TUI rendering
- **Web**: JavaScript/TypeScript with WASM core for simulation
- **Input Mapping**: Platform-specific controls mapped to standard input format
- **Rendering**: Client-specific rendering of shared game state

### Network Protocol

```
Tick Timeline:
T0: ┌─ Local Input  ─┐    ┌─ Remote Input ─┐
    │   axis_y: 50   │    │   axis_y: -30  │
    │   buttons: 0   │ ►► │   buttons: 1   │ ►►  Step Simulation
    └────────────────┘    └────────────────┘         │
                                                     ▼
T1: ┌─ Local Input  ─┐    ┌─ Remote Input ─┐    ┌─ Game State  ─┐
    │   axis_y: 25   │    │   axis_y: 0    │    │  paddles[2]   │
    │   buttons: 0   │ ►► │   buttons: 0   │ ►► │  ball: (x,y)  │
    └────────────────┘    └────────────────┘    │  score: [1,0] │
                                                └───────────────┘
```

### Deterministic Design Principles

1. **Fixed-Point Mathematics**: All calculations use 16.16 fixed-point format
2. **Reproducible Random**: Seeded PRNG for consistent ball serves
3. **Tick-Based Simulation**: No wall-clock time dependencies
4. **Input Quantization**: Analog inputs mapped to discrete values
5. **State Snapshots**: Complete game state serializable for sync

## 📊 Data Model

### Core Types

#### Fixed-Point Mathematics

```rust
pub type Fx = i32;                    // 16.16 fixed-point format
pub const FX_ONE: Fx = 1 << 16;      // Represents 1.0

// Utility functions
fx::from_f32(1.5) → 98304            // Convert from float
fx::to_f32(FX_ONE) → 1.0             // Convert to float
fx::mul_fx(a, b) → result            // Fixed-point multiply
```

#### Game Configuration

```rust
pub struct Config {
    pub paddle_half_h: Fx,            // Half-height of paddles
    pub paddle_speed: Fx,             // Movement speed per tick
    pub ball_speed: Fx,               // Initial ball speed
    pub ball_speed_up: Fx,            // Speed multiplier on hit
    pub wall_thickness: Fx,           // Wall collision thickness
    pub paddle_x: Fx,                 // Distance from screen edge
    pub max_score: u8,                // Score to win game
    pub seed: u64,                    // Random number seed
    pub tick_hz: u16,                 // Simulation frequency
    pub ball_radius: Fx,              // Ball collision radius
    pub paddle_width: Fx,             // Paddle collision width
}
```

#### Game State

```rust
pub struct Game {
    pub config: Config,               // Game configuration
    pub tick: Tick,                   // Current simulation tick
    pub status: Status,               // Game phase
    pub paddles: [Paddle; 2],         // Left and right paddles
    pub ball: Ball,                   // Ball state
    pub score: [u8; 2],               // [left, right] scores
    pub rng: u64,                     // Random state
}

pub enum Status {
    Lobby,                            // Waiting for ready
    Countdown(u16),                   // Countdown to start
    Playing,                          // Active gameplay
    Scored(Side, u16),                // Post-goal pause
    GameOver(Side),                   // Game finished
}
```

#### Input System

```rust
pub struct Input {
    pub axis_y: i8,                   // Vertical input [-127, 127]
    pub buttons: u8,                  // Button bitfield
}

pub struct InputPair {
    pub tick: Tick,                   // Target simulation tick
    pub a: Input,                     // Left player input
    pub b: Input,                     // Right player input
}
```

#### Physics Objects

```rust
pub struct Vec2 {
    pub x: Fx,
    pub y: Fx,
}

pub struct Paddle {
    pub y: Fx,                        // Center Y position
    pub vy: Fx,                       // Y velocity
}

pub struct Ball {
    pub pos: Vec2,                    // Position
    pub vel: Vec2,                    // Velocity
}
```

### Serialization Format

#### Input Pair (9 bytes) - ✅ Implemented

```
InputPair::encode() -> [u8; 9]  // Basic serialization without wire protocol headers
```

#### Snapshot (49 bytes) - ✅ Implemented

```
Snapshot::encode() -> Vec<u8>   // Binary format without wire protocol headers
```

#### [TODO] Wire Protocol Messages (not implemented)

```
[0x01][tick:u32][a_axis:i8][a_btn:u8][b_axis:i8][b_btn:u8]  // Input message
[0x02][tick:u32][status][paddles][ball][score][rng:u64]     // Snapshot message
[0x03][timestamp:u32]                                        // Ping message
```

### State Transitions

```
Game State Flow:

    ┌───────┐  both ready  ┌─────────────┐   timer expires  ┌─────────┐
    │ Lobby │ ──────────►  │ Countdown   │ ──────────────►  │ Playing │
    └───────┘              │ (180 ticks) │                  └─────────┘
        ▲                  └─────────────┘                       │
        │                                                        │ ball exit
        │                        ┌──────────────┐                │
        │  game over             │ Scored       │ ◄──────────────┘
        │                        │ (180 ticks)  │
        │                        └──────────────┘
        │                                 │
        │  max score reached              │ timer expires
        │                                 ▼
        │                        ┌──────────────┐
        └────────────────────────│  GameOver    │
                                 └──────────────┘
```

## 📚 API Documentation

### pong_core Public API

#### Game Management

```rust
impl Game {
    /// Create new game with configuration
    pub fn new(config: Config) -> Self;

    /// Step simulation forward one tick
    pub fn step(&mut self, inputs: &InputPair) -> Option<Event>;

    /// Get current game view for rendering
    pub fn view(&self) -> View;

    /// Create state snapshot
    pub fn snapshot(&self) -> Snapshot;

    /// Restore from snapshot
    pub fn restore(&mut self, snapshot: &Snapshot);

    /// Reset for new match
    pub fn reset_match(&mut self);

    /// Check if game accepts input
    pub fn is_active(&self) -> bool;

    /// Get winner if game over
    pub fn winner(&self) -> Option<Side>;
}
```

#### Input Creation

```rust
impl Input {
    /// Create new input
    pub fn new(axis_y: i8, buttons: u8) -> Self;

    /// Create zero input (no movement)
    pub fn zero() -> Self;

    /// Check if ready button pressed
    pub fn is_ready(&self) -> bool;
}

impl InputPair {
    /// Create input pair for specific tick
    pub fn new(tick: Tick, a: Input, b: Input) -> Self;

    /// Get input for specific side
    pub fn get_input(&self, side: Side) -> Input;
}
```

#### Fixed-Point Utilities

```rust
pub mod fx {
    /// Convert from f32 to fixed-point
    pub fn from_f32(f: f32) -> Fx;

    /// Convert from fixed-point to f32
    pub fn to_f32(value: Fx) -> f32;

    /// Multiply two fixed-point numbers
    pub fn mul_fx(a: Fx, b: Fx) -> Fx;

    /// Divide two fixed-point numbers
    pub fn div_fx(a: Fx, b: Fx) -> Fx;

    /// Clamp value between min and max
    pub fn clamp_fx(value: Fx, min: Fx, max: Fx) -> Fx;
}
```

### WASM Bindings API

#### WasmGame Interface

```rust
#[wasm_bindgen]
impl WasmGame {
    /// Create new game from JSON config
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<WasmGame, JsValue>;

    /// Step simulation with inputs
    #[wasm_bindgen]
    pub fn step(&mut self, tick: u32, a_axis: i8, a_btn: u8,
                b_axis: i8, b_btn: u8) -> Option<String>;

    /// Get game view as JSON
    #[wasm_bindgen]
    pub fn view_json(&self) -> String;

    /// Get snapshot as bytes
    #[wasm_bindgen]
    pub fn snapshot_bytes(&self) -> Vec<u8>;

    /// Restore from snapshot bytes
    #[wasm_bindgen]
    pub fn restore_bytes(&mut self, bytes: &[u8]);

    /// Reset the game to initial state (for rematch)
    #[wasm_bindgen]
    pub fn reset_match(&mut self);

    /// Get the current tick number
    #[wasm_bindgen]
    pub fn get_tick(&self) -> u32;

    /// Check if the game is currently active (accepting inputs)
    #[wasm_bindgen]
    pub fn is_active(&self) -> bool;

    /// Get a human-readable status string
    #[wasm_bindgen]
    pub fn status_string(&self) -> String;
}

/// Create a default config as JSON string (utility for JavaScript)
#[wasm_bindgen]
pub fn default_config_json() -> String;
```

### Usage Examples

#### Basic Game Loop

```rust
use pong_core::{Game, Config, Input, InputPair};

let mut game = Game::new(Config::default());
let mut tick = 0;

loop {
    // Get local input (example: keyboard)
    let local_input = Input::new(get_axis_input(), get_button_input());
    let remote_input = receive_remote_input(); // From network

    let inputs = InputPair::new(tick, local_input, remote_input);

    // Step simulation
    if let Some(event) = game.step(&inputs) {
        handle_game_event(event);
    }

    // Render current state
    let view = game.view();
    render_game(&view);

    tick += 1;
    std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
}
```

#### Fixed-Point Math

```rust
use pong_core::{fx, FX_ONE};

// Convert from floating point
let speed = fx::from_f32(1.5);           // 1.5 units/second
let half_field = FX_ONE / 2;             // 0.5 (field center)

// Physics calculation
let position = fx::mul_fx(speed, time);   // speed * time
let clamped = fx::clamp_fx(position, 0, FX_ONE); // Keep in bounds

// Convert back to float for rendering
let screen_pos = fx::to_f32(clamped) * screen_width;
```

#### Snapshot System

```rust
// Create checkpoint
let checkpoint = game.snapshot();

// Simulate some ticks
for i in 0..100 {
    let inputs = InputPair::new(tick + i, local_input, remote_input);
    game.step(&inputs);
}

// Restore if desynchronized
if needs_resync {
    game.restore(&checkpoint);
}
```

## 🤝 Contributing

We welcome contributions! Here's how to get started:

### Development Setup

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/project-pong.git`
3. Enter development environment: `devbox shell` (or install Rust manually)
4. Create a feature branch: `git checkout -b feature/amazing-feature`
5. Make changes and test: `cargo test`
6. Commit changes: `git commit -m 'Add amazing feature'`
7. Push to branch: `git push origin feature/amazing-feature`
8. Open a Pull Request

### Areas for Contribution

- **WebRTC Transport**: Implement P2P networking layer
- **Lockstep Networking**: Synchronization protocol for multiplayer
- **AI Improvements**: Enhance existing AI opponent intelligence
- **Visual Polish**: Enhanced rendering effects and animations
- **Performance**: Optimization and profiling for both native and WASM
- **Documentation**: Additional code examples and tutorials
- **Testing**: Expanded test coverage, especially integration tests
- **Mobile UX**: Further mobile experience improvements
- **Accessibility**: Screen reader support and keyboard navigation

### Code Review Guidelines

- Ensure all tests pass: `cargo test`
- Follow Rust naming conventions
- Document public APIs with `///` comments
- Maintain deterministic behavior in core
- No floating-point math in simulation code

### Reporting Issues

- Use GitHub Issues for bugs and feature requests
- Include minimal reproduction steps
- Specify platform and Rust version
- For performance issues, include profiling data

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🎯 Roadmap

### Current Status - Completed ✅

- ✅ **M1**: Core deterministic physics engine
- ✅ **M1**: Fixed-point mathematics system
- ✅ **M1**: Terminal client with TUI
- ✅ **M1**: CLI testing harness
- ✅ **M1**: Comprehensive test suite
- ✅ **M2**: Web client with WASM integration
- ✅ **M2**: TypeScript/Vite build system
- ✅ **M3**: Mobile-friendly touch controls
- ✅ **M3**: Responsive web design
- ✅ **M4**: AI opponent implementation
- ✅ **M4**: Local game modes (AI, wall, multiplayer)

### In Progress / Planned Features

- **M5**: WebRTC transport implementation ([TODO])
- **M5**: Lockstep networking protocol ([TODO])
- **M5**: P2P connectivity and signaling ([TODO])
- **M6**: Spectator mode and replays
- **M7**: Enhanced visual effects and animations
- **M8**: Performance optimizations and profiling

### Long-term Goals

- Tournament bracket system
- Custom game modes and physics
- Replay system with sharing
- Cross-platform leaderboards
- Plugin system for mods

---

**Built with ❤️ in Rust** | [Design Specification](design-spec.md) | [GitHub Repository](https://github.com/asyed94/project-pong)
