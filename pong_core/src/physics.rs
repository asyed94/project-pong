//! Deterministic physics engine for Pong.

use crate::types::{fx, *};

/// Physics calculations for game simulation
pub struct Physics;

impl Physics {
    /// Update paddle position based on input and constraints
    pub fn update_paddle(paddle: &mut Paddle, input: &Input, config: &Config) {
        // Convert input axis to velocity
        let target_velocity = if input.axis_y == 0 {
            0
        } else {
            // Convert [-127, 127] to fixed-point [-1.0, 1.0], then scale by paddle_speed
            let normalized_input = (input.axis_y as i32 * FX_ONE) / 127;
            fx::mul_fx(normalized_input, config.paddle_speed)
        };

        paddle.vy = target_velocity;

        // Update position using proper fixed-point division
        paddle.y += fx::div_fx(paddle.vy, config.tick_hz as i32 * FX_ONE);

        // Constrain paddle to field bounds
        let half_h = config.paddle_half_h;
        let min_y = half_h;
        let max_y = FX_ONE - half_h;

        paddle.y = fx::clamp_fx(paddle.y, min_y, max_y);

        // Stop velocity if we hit bounds
        if paddle.y <= min_y || paddle.y >= max_y {
            paddle.vy = 0;
        }
    }

    /// Update ball position and handle wall collisions
    pub fn update_ball(ball: &mut Ball, config: &Config) {
        // Update position using proper fixed-point division
        ball.pos.x += fx::div_fx(ball.vel.x, config.tick_hz as i32 * FX_ONE);
        ball.pos.y += fx::div_fx(ball.vel.y, config.tick_hz as i32 * FX_ONE);

        // Handle top/bottom wall collisions
        if ball.pos.y <= 0 {
            ball.pos.y = 0;
            ball.vel.y = -ball.vel.y; // Reverse Y velocity
        } else if ball.pos.y >= FX_ONE {
            ball.pos.y = FX_ONE;
            ball.vel.y = -ball.vel.y; // Reverse Y velocity
        }
    }

    /// Check for paddle-ball collision and handle it
    pub fn check_paddle_collision(
        ball: &mut Ball,
        paddle: &Paddle,
        side: Side,
        config: &Config,
    ) -> bool {
        let paddle_x = match side {
            Side::Left => config.paddle_x,
            Side::Right => FX_ONE - config.paddle_x,
        };

        // Simple rectangular collision detection using proper fixed-point math
        let ball_radius = fx::div_fx(config.ball_speed, 60 * FX_ONE); // Small radius based on speed
        let paddle_half_h = config.paddle_half_h;

        let ball_left = ball.pos.x - ball_radius;
        let ball_right = ball.pos.x + ball_radius;
        let ball_top = ball.pos.y - ball_radius;
        let ball_bottom = ball.pos.y + ball_radius;

        let paddle_left = paddle_x - ball_radius;
        let paddle_right = paddle_x + ball_radius;
        let paddle_top = paddle.y - paddle_half_h;
        let paddle_bottom = paddle.y + paddle_half_h;

        // Check if ball overlaps with paddle
        if ball_right >= paddle_left
            && ball_left <= paddle_right
            && ball_bottom >= paddle_top
            && ball_top <= paddle_bottom
        {
            // Collision detected - reflect ball
            match side {
                Side::Left => {
                    if ball.vel.x < 0 {
                        // Only reflect if moving toward paddle
                        ball.vel.x = -ball.vel.x;
                        ball.pos.x = paddle_right + ball_radius; // Push ball away

                        // Add paddle velocity influence using proper fixed-point division
                        let velocity_influence = fx::div_fx(paddle.vy, 4 * FX_ONE); // Reduce influence
                        ball.vel.y += velocity_influence;

                        // Speed up ball
                        ball.vel.x = fx::mul_fx(ball.vel.x, config.ball_speed_up);
                        ball.vel.y = fx::mul_fx(ball.vel.y, config.ball_speed_up);

                        return true;
                    }
                }
                Side::Right => {
                    if ball.vel.x > 0 {
                        // Only reflect if moving toward paddle
                        ball.vel.x = -ball.vel.x;
                        ball.pos.x = paddle_left - ball_radius; // Push ball away

                        // Add paddle velocity influence using proper fixed-point division
                        let velocity_influence = fx::div_fx(paddle.vy, 4 * FX_ONE); // Reduce influence
                        ball.vel.y += velocity_influence;

                        // Speed up ball
                        ball.vel.x = fx::mul_fx(ball.vel.x, config.ball_speed_up);
                        ball.vel.y = fx::mul_fx(ball.vel.y, config.ball_speed_up);

                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if ball is out of bounds (scoring condition)
    pub fn check_scoring(ball: &Ball) -> Option<Side> {
        if ball.pos.x < 0 {
            Some(Side::Right) // Right player scored
        } else if ball.pos.x > FX_ONE {
            Some(Side::Left) // Left player scored
        } else {
            None
        }
    }

    /// Reset ball for serve
    pub fn serve_ball(ball: &mut Ball, serving_side: Side, config: &Config, rng_state: &mut u64) {
        // Center the ball
        ball.pos = Vec2::new(FX_ONE / 2, FX_ONE / 2);

        // Generate serve direction with some randomness
        let base_speed = config.ball_speed;

        // Simple linear congruential generator for deterministic randomness
        *rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let random_angle = (*rng_state >> 16) as i32;

        // Convert to Y velocity component (roughly -30° to +30°)
        let y_vel = (random_angle % (FX_ONE / 2)) - (FX_ONE / 4);

        // X velocity based on serving side
        let x_vel = match serving_side {
            Side::Left => base_speed,   // Serve to right
            Side::Right => -base_speed, // Serve to left
        };

        ball.vel = Vec2::new(x_vel, y_vel);
    }

    /// Limit ball speed to prevent runaway velocity
    pub fn limit_ball_speed(ball: &mut Ball, max_speed: Fx) {
        let speed_squared = fx::mul_fx(ball.vel.x, ball.vel.x) + fx::mul_fx(ball.vel.y, ball.vel.y);
        let max_speed_squared = fx::mul_fx(max_speed, max_speed);

        if speed_squared > max_speed_squared {
            // Calculate current speed
            let current_speed = Self::sqrt_fx(speed_squared);
            let scale = fx::div_fx(max_speed, current_speed);

            ball.vel.x = fx::mul_fx(ball.vel.x, scale);
            ball.vel.y = fx::mul_fx(ball.vel.y, scale);
        }
    }

    /// Fixed-point square root approximation using Newton's method
    fn sqrt_fx(value: Fx) -> Fx {
        if value <= 0 {
            return 0;
        }

        let mut x = value;
        let mut prev_x;

        // Newton's method: x_new = (x + value/x) / 2
        for _ in 0..10 {
            // Limit iterations
            prev_x = x;
            x = (x + fx::div_fx(value, x)) / 2;

            // Check for convergence
            if fx::abs_fx(x - prev_x) < 16 {
                // Small threshold
                break;
            }
        }

        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paddle_movement() {
        let config = Config::default();
        let mut paddle = Paddle::new(FX_ONE / 2);

        // Test upward movement
        let input_up = Input::new(127, 0); // Maximum up
        Physics::update_paddle(&mut paddle, &input_up, &config);

        assert!(paddle.vy > 0); // Should have positive velocity

        // Test downward movement
        let input_down = Input::new(-127, 0); // Maximum down
        Physics::update_paddle(&mut paddle, &input_down, &config);

        assert!(paddle.vy < 0); // Should have negative velocity

        // Test no input
        let input_none = Input::zero();
        Physics::update_paddle(&mut paddle, &input_none, &config);

        assert_eq!(paddle.vy, 0); // Should have zero velocity
    }

    #[test]
    fn test_paddle_bounds() {
        let config = Config::default();
        let mut paddle = Paddle::new(0); // Start at bottom

        // Try to move below bottom
        let input_down = Input::new(-127, 0);
        Physics::update_paddle(&mut paddle, &input_down, &config);

        assert!(paddle.y >= config.paddle_half_h); // Should be constrained

        // Test top bound
        paddle.y = FX_ONE; // Start at top
        let input_up = Input::new(127, 0);
        Physics::update_paddle(&mut paddle, &input_up, &config);

        assert!(paddle.y <= FX_ONE - config.paddle_half_h); // Should be constrained
    }

    #[test]
    fn test_ball_wall_collision() {
        let config = Config::default();
        let mut ball = Ball::new(
            Vec2::new(FX_ONE / 2, 0),  // At bottom wall
            Vec2::new(0, -FX_ONE / 4), // Moving down
        );

        Physics::update_ball(&mut ball, &config);

        assert_eq!(ball.pos.y, 0); // Should be at wall
        assert!(ball.vel.y > 0); // Velocity should reverse
    }

    #[test]
    fn test_scoring_detection() {
        // Ball past left edge
        let ball_left = Ball::new(Vec2::new(-FX_ONE / 4, FX_ONE / 2), Vec2::zero());
        assert_eq!(Physics::check_scoring(&ball_left), Some(Side::Right));

        // Ball past right edge
        let ball_right = Ball::new(Vec2::new(FX_ONE + FX_ONE / 4, FX_ONE / 2), Vec2::zero());
        assert_eq!(Physics::check_scoring(&ball_right), Some(Side::Left));

        // Ball in bounds
        let ball_center = Ball::new(Vec2::new(FX_ONE / 2, FX_ONE / 2), Vec2::zero());
        assert_eq!(Physics::check_scoring(&ball_center), None);
    }

    #[test]
    fn test_serve_ball() {
        let config = Config::default();
        let mut ball = Ball::new(Vec2::zero(), Vec2::zero());
        let mut rng = 12345u64;

        Physics::serve_ball(&mut ball, Side::Left, &config, &mut rng);

        // Ball should be centered
        assert_eq!(ball.pos.x, FX_ONE / 2);
        assert_eq!(ball.pos.y, FX_ONE / 2);

        // Should have rightward velocity when left serves
        assert!(ball.vel.x > 0);

        // Test right serve
        Physics::serve_ball(&mut ball, Side::Right, &config, &mut rng);
        assert!(ball.vel.x < 0); // Should have leftward velocity
    }

    #[test]
    fn test_ball_speed_limiting() {
        let max_speed = FX_ONE * 2; // 2.0 units/s

        let mut ball = Ball::new(
            Vec2::zero(),
            Vec2::new(FX_ONE * 4, FX_ONE * 4), // Very fast
        );

        Physics::limit_ball_speed(&mut ball, max_speed);

        // Speed should be reduced
        let final_speed_sq =
            fx::mul_fx(ball.vel.x, ball.vel.x) + fx::mul_fx(ball.vel.y, ball.vel.y);
        let max_speed_sq = fx::mul_fx(max_speed, max_speed);

        assert!(final_speed_sq <= max_speed_sq + 1000); // Allow small tolerance
    }

    #[test]
    fn test_paddle_collision() {
        let config = Config::default();
        let paddle = Paddle::new(FX_ONE / 2); // Center paddle

        let mut ball = Ball::new(
            Vec2::new(config.paddle_x, FX_ONE / 2), // At paddle position
            Vec2::new(-FX_ONE / 4, 0),              // Moving toward left paddle
        );

        let hit = Physics::check_paddle_collision(&mut ball, &paddle, Side::Left, &config);

        assert!(hit); // Should detect collision
        assert!(ball.vel.x > 0); // Ball should reverse direction
    }

    #[test]
    fn test_sqrt_fx() {
        // Test some known values
        assert_eq!(Physics::sqrt_fx(FX_ONE), FX_ONE); // sqrt(1) = 1
        assert_eq!(Physics::sqrt_fx(FX_ONE * 4), FX_ONE * 2); // sqrt(4) = 2

        // Test with small values - sqrt(0.25) = 0.5
        let quarter = FX_ONE / 4;
        let sqrt_quarter = Physics::sqrt_fx(quarter);
        let expected = FX_ONE / 2; // 0.5
                                   // Allow small tolerance for fixed-point precision
        assert!(
            fx::abs_fx(sqrt_quarter - expected) < 1000,
            "sqrt(0.25) = {} but expected ~{}",
            fx::to_f32(sqrt_quarter),
            fx::to_f32(expected)
        );
    }

    #[test]
    fn test_deterministic_rng() {
        let _config = Config::default();
        let mut ball1 = Ball::new(Vec2::zero(), Vec2::zero());
        let mut ball2 = Ball::new(Vec2::zero(), Vec2::zero());
        let mut rng1 = 12345u64;
        let mut rng2 = 12345u64;

        Physics::serve_ball(&mut ball1, Side::Left, &_config, &mut rng1);
        Physics::serve_ball(&mut ball2, Side::Left, &_config, &mut rng2);

        // Same seed should produce identical results
        assert_eq!(ball1.vel, ball2.vel);
        assert_eq!(rng1, rng2);
    }
}
