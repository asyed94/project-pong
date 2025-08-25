//! Deterministic P2P Pong Game Engine
//!
//! This crate provides a deterministic game engine for Pong that uses fixed-point
//! arithmetic to ensure identical behavior across different platforms and network peers.
//!
//! # Features
//!
//! - **Deterministic**: Uses 16.16 fixed-point math for consistent results
//! - **Tick-based**: Pure function simulation driven by discrete ticks
//! - **Serializable**: Compact binary serialization for networking
//! - **Cross-platform**: No external dependencies, works on native and WASM
//!
//! # Basic Usage
//!
//! ```rust
//! use pong_core::*;
//!
//! // Create a new game with default settings
//! let config = Config::default();
//! let mut game = Game::new(config);
//!
//! // Create input for both players
//! let inputs = InputPair::new(
//!     game.view().tick,
//!     Input::new(50, 0),   // Left player: move up
//!     Input::new(-30, 0),  // Right player: move down
//! );
//!
//! // Step the simulation forward
//! if let Some(event) = game.step(&inputs) {
//!     match event {
//!         Event::Scored { scorer, score } => {
//!             println!("Player {:?} scored! Score: {:?}", scorer, score);
//!         }
//!     }
//! }
//!
//! // Get view for rendering
//! let view = game.view();
//! println!("Ball at ({}, {})",
//!     fx::to_f32(view.ball_pos.x),
//!     fx::to_f32(view.ball_pos.y)
//! );
//! ```

pub mod game;
pub mod physics;
pub mod serialization;
pub mod types;

// Re-export all public types for convenience
pub use game::Game;
pub use serialization::SerializationError;
pub use types::*;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test: Full game simulation from lobby to game over
    #[test]
    fn test_full_game_flow() {
        let mut config = Config::default();
        config.max_score = 3; // Shorter game for testing

        let mut game = Game::new(config);
        let mut tick = 0;

        // Test lobby phase - both players ready
        let ready_inputs = InputPair::new(tick, Input::new(0, 1), Input::new(0, 1));
        let event = game.step(&ready_inputs);
        assert!(event.is_none());
        assert!(matches!(game.status, Status::Countdown(_)));
        tick += 1;

        // Skip through countdown
        while matches!(game.status, Status::Countdown(_)) {
            let inputs = InputPair::new(tick, Input::zero(), Input::zero());
            game.step(&inputs);
            tick += 1;
        }

        assert_eq!(game.status, Status::Playing);

        // Simulate some gameplay with alternating strong inputs
        let mut events = Vec::new();
        for i in 0..3000 {
            // Increase limit for completion
            if matches!(game.status, Status::GameOver(_)) {
                break;
            }

            // Use alternating strong inputs to create dynamic gameplay
            let inputs = if i % 120 < 60 {
                // Phase 1: Left paddle up, right paddle down
                InputPair::new(tick, Input::new(100, 0), Input::new(-100, 0))
            } else {
                // Phase 2: Left paddle down, right paddle up
                InputPair::new(tick, Input::new(-100, 0), Input::new(100, 0))
            };

            if let Some(event) = game.step(&inputs) {
                events.push(event);
            }
            tick += 1;
        }

        // Should have some scoring events
        assert!(!events.is_empty());

        // Game should eventually end
        assert!(matches!(game.status, Status::GameOver(_)));

        // Someone should have won
        assert!(game.winner().is_some());

        // Score should be at max
        assert!(game.score[0] >= config.max_score || game.score[1] >= config.max_score);
    }

    /// Test serialization round-trip consistency
    #[test]
    fn test_serialization_roundtrip() {
        // Test Input
        let input = Input::new(-100, 5);
        let input_bytes = input.encode();
        let decoded_input = Input::decode(&input_bytes).unwrap();
        assert_eq!(input, decoded_input);

        // Test InputPair
        let pair = InputPair::new(12345, Input::new(-50, 1), Input::new(75, 2));
        let pair_bytes = pair.encode();
        let decoded_pair = InputPair::decode(&pair_bytes).unwrap();
        assert_eq!(pair, decoded_pair);

        // Test Snapshot
        let game = Game::new(Config::default());
        let snapshot = game.snapshot();
        let snapshot_bytes = snapshot.encode();
        let decoded_snapshot = Snapshot::decode(&snapshot_bytes).unwrap();

        // Verify all fields match
        assert_eq!(snapshot.tick, decoded_snapshot.tick);
        assert_eq!(snapshot.status, decoded_snapshot.status);
        assert_eq!(snapshot.paddles, decoded_snapshot.paddles);
        assert_eq!(snapshot.ball.pos, decoded_snapshot.ball.pos);
        assert_eq!(snapshot.ball.vel, decoded_snapshot.ball.vel);
        assert_eq!(snapshot.score, decoded_snapshot.score);
        assert_eq!(snapshot.rng, decoded_snapshot.rng);
    }

    /// Test deterministic behavior across game instances
    #[test]
    fn test_deterministic_games() {
        let config = Config::default();
        let mut game1 = Game::new(config);
        let mut game2 = Game::new(config);

        // Apply identical input sequences
        let input_sequence = [
            InputPair::new(0, Input::new(0, 1), Input::new(0, 1)), // Ready
            InputPair::new(1, Input::new(50, 0), Input::new(-30, 0)),
            InputPair::new(2, Input::new(-20, 0), Input::new(75, 0)),
            InputPair::new(3, Input::new(0, 0), Input::new(0, 0)),
            InputPair::new(4, Input::new(127, 0), Input::new(-127, 0)),
        ];

        let mut events1 = Vec::new();
        let mut events2 = Vec::new();

        for inputs in &input_sequence {
            if let Some(event) = game1.step(inputs) {
                events1.push(event);
            }
            if let Some(event) = game2.step(inputs) {
                events2.push(event);
            }
        }

        // Both games should have identical state
        assert_eq!(game1.tick, game2.tick);
        assert_eq!(game1.status, game2.status);
        assert_eq!(game1.paddles, game2.paddles);
        assert_eq!(game1.ball.pos, game2.ball.pos);
        assert_eq!(game1.ball.vel, game2.ball.vel);
        assert_eq!(game1.score, game2.score);
        assert_eq!(game1.rng, game2.rng);

        // Both games should have identical events
        assert_eq!(events1, events2);

        // Views should be identical
        let view1 = game1.view();
        let view2 = game2.view();
        assert_eq!(view1.tick, view2.tick);
        assert_eq!(view1.status, view2.status);
        assert_eq!(view1.left_y, view2.left_y);
        assert_eq!(view1.right_y, view2.right_y);
        assert_eq!(view1.ball_pos, view2.ball_pos);
        assert_eq!(view1.score, view2.score);
    }

    /// Test snapshot restore preserves exact game state
    #[test]
    fn test_snapshot_state_preservation() {
        let mut game = Game::new(Config::default());

        // Advance game to interesting state
        let inputs = [
            InputPair::new(0, Input::new(0, 1), Input::new(0, 1)), // Ready
            InputPair::new(1, Input::new(50, 0), Input::new(-30, 0)),
            InputPair::new(2, Input::new(-20, 0), Input::new(75, 0)),
        ];

        for input in &inputs {
            game.step(input);
        }

        // Create snapshot
        let snapshot = game.snapshot();

        // Continue simulation
        let more_inputs = [
            InputPair::new(3, Input::new(100, 0), Input::new(-100, 0)),
            InputPair::new(4, Input::new(0, 0), Input::new(50, 0)),
        ];

        for input in &more_inputs {
            game.step(input);
        }

        // State should be different now
        let current_snapshot = game.snapshot();
        assert_ne!(snapshot.tick, current_snapshot.tick);

        // Restore original snapshot
        game.restore(&snapshot);

        // Should be back to original state
        let restored_snapshot = game.snapshot();
        assert_eq!(snapshot.tick, restored_snapshot.tick);
        assert_eq!(snapshot.status, restored_snapshot.status);
        assert_eq!(snapshot.paddles, restored_snapshot.paddles);
        assert_eq!(snapshot.ball.pos, restored_snapshot.ball.pos);
        assert_eq!(snapshot.ball.vel, restored_snapshot.ball.vel);
        assert_eq!(snapshot.score, restored_snapshot.score);
        assert_eq!(snapshot.rng, restored_snapshot.rng);

        // Continue from restored state should be deterministic
        game.step(&more_inputs[0]);
        let view_after_restore = game.view();

        // Create another game and replay from snapshot
        let mut game2 = Game::new(Config::default());
        game2.restore(&snapshot);
        game2.step(&more_inputs[0]);
        let view2 = game2.view();

        // Should be identical
        assert_eq!(view_after_restore.tick, view2.tick);
        assert_eq!(view_after_restore.ball_pos, view2.ball_pos);
        assert_eq!(view_after_restore.left_y, view2.left_y);
        assert_eq!(view_after_restore.right_y, view2.right_y);
    }

    /// Test edge cases in physics
    #[test]
    fn test_physics_edge_cases() {
        let mut game = Game::new(Config::default());
        game.status = Status::Playing;

        // Test ball at exact boundaries
        game.ball.pos.y = 0; // Bottom wall
        game.ball.vel.y = -1000; // Moving down fast

        let inputs = InputPair::new(0, Input::zero(), Input::zero());
        game.step(&inputs);

        // Ball should bounce off bottom wall
        assert_eq!(game.ball.pos.y, 0);
        assert!(game.ball.vel.y > 0); // Should be moving up now

        // Test ball at top wall
        game.ball.pos.y = FX_ONE; // Top wall
        game.ball.vel.y = 1000; // Moving up fast

        let inputs = InputPair::new(1, Input::zero(), Input::zero());
        game.step(&inputs);

        // Ball should bounce off top wall
        assert_eq!(game.ball.pos.y, FX_ONE);
        assert!(game.ball.vel.y < 0); // Should be moving down now
    }

    /// Test input validation and edge cases
    #[test]
    fn test_input_edge_cases() {
        let mut game = Game::new(Config::default());
        game.status = Status::Playing;

        // Test extreme input values
        let extreme_inputs = InputPair::new(
            0,
            Input::new(127, 255), // Max positive
            Input::new(-127, 0),  // Max negative
        );

        let initial_left_y = game.paddles[0].y;
        let initial_right_y = game.paddles[1].y;

        game.step(&extreme_inputs);

        // Paddles should move according to input
        assert_ne!(game.paddles[0].y, initial_left_y);
        assert_ne!(game.paddles[1].y, initial_right_y);

        // Paddles should stay within bounds
        let half_h = game.config.paddle_half_h;
        assert!(game.paddles[0].y >= half_h);
        assert!(game.paddles[0].y <= FX_ONE - half_h);
        assert!(game.paddles[1].y >= half_h);
        assert!(game.paddles[1].y <= FX_ONE - half_h);
    }
}
