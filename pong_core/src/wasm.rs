//! WASM bridge module for JavaScript interop

use crate::{Config, Game, Input, InputPair};
use wasm_bindgen::prelude::*;

// Console logging placeholder (can be implemented later)
macro_rules! console_log {
    ($($t:tt)*) => {};
}

/// WASM-compatible wrapper around the core Game engine
#[wasm_bindgen]
pub struct WasmGame {
    inner: Game,
}

#[wasm_bindgen]
impl WasmGame {
    /// Create a new game instance from JSON configuration
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: String) -> Result<WasmGame, JsValue> {
        // For now, use default config - later we can deserialize JSON config
        let config = if config_json.trim().is_empty() {
            Config::default()
        } else {
            // Try to deserialize from JSON, fall back to default on error
            match serde_json::from_str(&config_json) {
                Ok(config) => config,
                Err(e) => {
                    console_log!("Failed to parse config JSON, using defaults: {}", e);
                    Config::default()
                }
            }
        };

        let game = Game::new(config);
        console_log!("WasmGame created with tick_hz: {}", config.tick_hz);

        Ok(WasmGame { inner: game })
    }

    /// Step the game forward one tick with inputs for both players
    /// Returns JSON-serialized Event or null if no event occurred
    pub fn step(
        &mut self,
        tick: u32,
        a_axis: i8,
        a_btn: u8,
        b_axis: i8,
        b_btn: u8,
    ) -> Option<String> {
        let input_a = Input::new(a_axis, a_btn);
        let input_b = Input::new(b_axis, b_btn);
        let input_pair = InputPair::new(tick, input_a, input_b);

        if let Some(event) = self.inner.step(&input_pair) {
            // Serialize the event to JSON
            match serde_json::to_string(&event) {
                Ok(json) => Some(json),
                Err(e) => {
                    console_log!("Failed to serialize event: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Get the current game view as JSON string
    pub fn view_json(&self) -> String {
        let view = self.inner.view();
        match serde_json::to_string(&view) {
            Ok(json) => json,
            Err(e) => {
                console_log!("Failed to serialize view: {}", e);
                "{}".to_string() // Return empty object on error
            }
        }
    }

    /// Get a snapshot of the current game state as bytes
    pub fn snapshot_bytes(&self) -> Vec<u8> {
        let snapshot = self.inner.snapshot();
        snapshot.encode()
    }

    /// Restore game state from snapshot bytes
    pub fn restore_bytes(&mut self, bytes: &[u8]) {
        match crate::Snapshot::decode(bytes) {
            Ok(snapshot) => {
                self.inner.restore(&snapshot);
                console_log!("Game state restored from snapshot");
            }
            Err(e) => {
                console_log!("Failed to restore from snapshot: {:?}", e);
            }
        }
    }

    /// Reset the game to initial state (useful for rematch)
    pub fn reset_match(&mut self) {
        self.inner.reset_match();
        console_log!("Game reset for new match");
    }

    /// Get the current tick number
    pub fn get_tick(&self) -> u32 {
        self.inner.tick
    }

    /// Check if the game is currently active (accepting inputs)
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }

    /// Get a human-readable status string
    pub fn status_string(&self) -> String {
        self.inner.status_string().to_string()
    }
}

// Additional helper functions for WASM integration

/// Create a default config as JSON string (utility for JavaScript)
#[wasm_bindgen]
pub fn default_config_json() -> String {
    let config = Config::default();
    match serde_json::to_string_pretty(&config) {
        Ok(json) => json,
        Err(_) => "{}".to_string(),
    }
}

/// Initialize WASM module (called automatically)
#[wasm_bindgen(start)]
pub fn init() {
    console_log!("Pong WASM module initialized!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_game_creation() {
        let game = WasmGame::new("{}".to_string()).unwrap();
        assert_eq!(game.get_tick(), 0);
        assert!(!game.is_active()); // Should be in lobby
    }

    #[test]
    fn test_wasm_game_step() {
        let mut game = WasmGame::new("{}".to_string()).unwrap();

        // Step with ready inputs to start countdown
        let event = game.step(0, 0, 1, 0, 1); // Both players ready
        assert!(event.is_none()); // Should not be an event, just state change

        // Check that the game state advanced
        assert_eq!(game.get_tick(), 1);
    }

    #[test]
    fn test_view_json_serialization() {
        let game = WasmGame::new("{}".to_string()).unwrap();
        let json = game.view_json();

        // Should be valid JSON
        assert!(!json.is_empty());
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));

        // Should be deserializable
        let view: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(view.get("tick").is_some());
        assert!(view.get("status").is_some());
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let mut game1 = WasmGame::new("{}".to_string()).unwrap();

        // Modify game state
        let _ = game1.step(0, 50, 1, -30, 1);

        // Take snapshot
        let snapshot_bytes = game1.snapshot_bytes();
        assert!(!snapshot_bytes.is_empty());

        // Restore to new game
        let mut game2 = WasmGame::new("{}".to_string()).unwrap();
        game2.restore_bytes(&snapshot_bytes);

        // Games should have same state
        assert_eq!(game1.get_tick(), game2.get_tick());
        assert_eq!(game1.view_json(), game2.view_json());
    }

    #[test]
    fn test_default_config_json() {
        let json = default_config_json();
        assert!(!json.is_empty());

        // Should be deserializable
        let _config: Config = serde_json::from_str(&json).unwrap();
    }
}
