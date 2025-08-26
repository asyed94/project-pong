//! Pong core game engine - deterministic multiplayer pong implementation

pub mod game;
pub mod physics;
pub mod serialization;
pub mod types;

// WASM bridge module - only compiled when wasm feature is enabled
#[cfg(feature = "wasm")]
pub mod wasm;

pub use game::Game;
pub use types::*;

#[cfg(test)]
mod paddle_height_tests {
    use super::*;

    #[test]
    fn test_paddle_height_consistency() {
        let config = Config::default();
        let mut game = Game::new(config);

        // Test field dimensions
        let field_width = 80;
        let field_height = 24;

        // Create RenderHelper for consistent rendering
        let render_helper = RenderHelper::new(field_width, field_height, &config);

        // Test paddle at different Y positions
        let test_positions = [
            fx::from_f32(0.1),  // Near bottom
            fx::from_f32(0.25), // Quarter up
            fx::from_f32(0.5),  // Center
            fx::from_f32(0.75), // Three quarters up
            fx::from_f32(0.9),  // Near top
        ];

        let mut paddle_heights = Vec::new();

        for &pos in &test_positions {
            // Move left paddle to test position
            game.paddles[0].y = pos;

            // Get paddle rectangle using RenderHelper
            let paddle_rect = render_helper.get_paddle_rect(pos, Side::Left);

            // Calculate paddle height in screen coordinates
            let paddle_height = paddle_rect.bottom - paddle_rect.top + 1;
            paddle_heights.push(paddle_height);

            println!(
                "Position: {:.2} -> Paddle height: {} pixels",
                fx::to_f32(pos),
                paddle_height
            );
        }

        // All paddle heights should be EXACTLY the same with new architecture
        let first_height = paddle_heights[0];
        for (i, &height) in paddle_heights.iter().enumerate() {
            assert_eq!(
                height,
                first_height,
                "Paddle height inconsistency at position {}: expected {}, got {}",
                fx::to_f32(test_positions[i]),
                first_height,
                height
            );
        }

        println!(
            "✓ All paddle heights are perfectly consistent: {} pixels",
            first_height
        );
    }

    #[test]
    fn test_paddle_boundaries() {
        let config = Config::default();
        let mut game = Game::new(config);

        let field_width = 80;
        let field_height = 24;
        let render_helper = RenderHelper::new(field_width, field_height, &config);

        // Test paddle at extreme positions to ensure bounds are respected
        let extreme_positions = [
            config.paddle_half_h,          // Minimum Y (paddle touching bottom)
            FX_ONE - config.paddle_half_h, // Maximum Y (paddle touching top)
        ];

        for &pos in &extreme_positions {
            game.paddles[0].y = pos;
            let paddle_rect = render_helper.get_paddle_rect(pos, Side::Left);

            // Paddle should stay within field bounds
            assert!(paddle_rect.top < field_height);
            assert!(paddle_rect.bottom < field_height);
            assert!(paddle_rect.top <= paddle_rect.bottom);

            println!(
                "Extreme position {:.2} -> top: {}, bottom: {}",
                fx::to_f32(pos),
                paddle_rect.top,
                paddle_rect.bottom
            );
        }

        println!("✓ Paddle boundaries are properly respected");
    }

    #[test]
    fn test_render_helper_consistency() {
        let config = Config::default();
        let mut game = Game::new(config);

        let field_width = 80;
        let field_height = 24;
        let render_helper = RenderHelper::new(field_width, field_height, &config);

        // Test with small incremental movements to verify perfect consistency
        let base_pos = fx::from_f32(0.5);
        let small_increment = FX_ONE / 100; // 0.01 in fixed-point

        let mut prev_height: Option<usize> = None;

        for i in 0..10 {
            let pos = base_pos + (small_increment * i);
            game.paddles[0].y = pos;

            let paddle_rect = render_helper.get_paddle_rect(pos, Side::Left);
            let height = paddle_rect.bottom - paddle_rect.top + 1;

            if let Some(prev) = prev_height {
                // Height should be EXACTLY the same with new architecture
                assert_eq!(
                    height,
                    prev,
                    "Paddle height changed unexpectedly at position {}: {} vs {}",
                    fx::to_f32(pos),
                    height,
                    prev
                );
            }

            prev_height = Some(height);
        }

        println!("✓ RenderHelper provides perfect consistency across all positions");
    }

    #[test]
    fn test_render_helper_fixed_dimensions() {
        let config = Config::default();
        let render_helper = RenderHelper::new(80, 24, &config);

        // The RenderHelper should report consistent paddle dimensions
        let expected_height = render_helper.paddle_height_pixels();

        // Test at various positions
        let test_positions = [
            fx::from_f32(0.1),
            fx::from_f32(0.3),
            fx::from_f32(0.5),
            fx::from_f32(0.7),
            fx::from_f32(0.9),
        ];

        for pos in test_positions {
            let left_rect = render_helper.get_paddle_rect(pos, Side::Left);
            let right_rect = render_helper.get_paddle_rect(pos, Side::Right);

            let left_height = left_rect.bottom - left_rect.top + 1;
            let right_height = right_rect.bottom - right_rect.top + 1;

            assert_eq!(left_height, expected_height);
            assert_eq!(right_height, expected_height);
            assert_eq!(left_height, right_height);
        }

        println!(
            "✓ RenderHelper maintains fixed paddle dimensions: {} pixels",
            expected_height
        );
    }
}
