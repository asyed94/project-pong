//! Core types and constants for the deterministic Pong game engine.

/// Fixed-point type using 16.16 format (16 integer bits, 16 fractional bits)
pub type Fx = i32;

/// One unit in fixed-point format
pub const FX_ONE: Fx = 1 << 16;

/// Tick counter type
pub type Tick = u32;

/// Player/paddle side
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub enum Side {
    Left,
    Right,
}

impl Side {
    /// Get the opposite side
    pub fn opposite(self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

/// Game status
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub enum Status {
    /// Waiting for players to be ready
    Lobby,
    /// Countdown before serve (ticks remaining)
    Countdown(u16),
    /// Active gameplay
    Playing,
    /// Someone scored (scorer, ticks until next serve)
    Scored(Side, u16),
    /// Game over (winner)
    GameOver(Side),
}

/// Game configuration
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    /// Half-height of paddle
    pub paddle_half_h: Fx,
    /// Paddle movement speed (units per tick)
    pub paddle_speed: Fx,
    /// Initial ball speed
    pub ball_speed: Fx,
    /// Speed multiplier on paddle hit
    pub ball_speed_up: Fx,
    /// Wall thickness (usually 0)
    pub wall_thickness: Fx,
    /// Paddle X position from edge
    pub paddle_x: Fx,
    /// Score to win
    pub max_score: u8,
    /// Random seed
    pub seed: u64,
    /// Tick frequency (Hz)
    pub tick_hz: u16,
    /// Fixed ball collision radius
    pub ball_radius: Fx,
    /// Paddle width for collision detection
    pub paddle_width: Fx,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            paddle_half_h: FX_ONE / 8,             // 1/8 unit = 8192 (exact)
            paddle_speed: FX_ONE * 3,              // 3.0 units/s = 196608 (exact)
            ball_speed: FX_ONE / 2,                // 0.5 units/s = 32768 (exact)
            ball_speed_up: FX_ONE + (FX_ONE / 20), // +5% per hit
            wall_thickness: 0,
            paddle_x: fx::from_f32(0.05), // 5% from edge (precise conversion)
            max_score: 11,
            seed: 0xC0FFEE,
            tick_hz: 60,
            ball_radius: fx::from_f32(1.0 / 32.0), // Precise small ball radius
            paddle_width: fx::from_f32(0.025),     // 2.5% width (precise conversion)
        }
    }
}

/// 2D vector in fixed-point
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub struct Vec2 {
    pub x: Fx,
    pub y: Fx,
}

impl Vec2 {
    pub fn new(x: Fx, y: Fx) -> Self {
        Vec2 { x, y }
    }

    pub fn zero() -> Self {
        Vec2 { x: 0, y: 0 }
    }
}

/// Paddle state
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Paddle {
    /// Y position (center)
    pub y: Fx,
    /// Y velocity
    pub vy: Fx,
}

impl Paddle {
    pub fn new(y: Fx) -> Self {
        Paddle { y, vy: 0 }
    }
}

/// Ball state
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ball {
    /// Position
    pub pos: Vec2,
    /// Velocity
    pub vel: Vec2,
}

impl Ball {
    pub fn new(pos: Vec2, vel: Vec2) -> Self {
        Ball { pos, vel }
    }
}

/// Player input for one tick
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Input {
    /// Vertical axis input [-127, 127]
    pub axis_y: i8,
    /// Button bitfield
    pub buttons: u8,
}

impl Input {
    pub fn new(axis_y: i8, buttons: u8) -> Self {
        Input { axis_y, buttons }
    }

    pub fn zero() -> Self {
        Input {
            axis_y: 0,
            buttons: 0,
        }
    }

    /// Check if ready button (bit 0) is pressed
    pub fn is_ready(&self) -> bool {
        (self.buttons & 1) != 0
    }
}

/// Input pair for both players on a specific tick
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InputPair {
    pub tick: Tick,
    pub a: Input, // Left player
    pub b: Input, // Right player
}

impl InputPair {
    pub fn new(tick: Tick, a: Input, b: Input) -> Self {
        InputPair { tick, a, b }
    }

    pub fn get_input(&self, side: Side) -> Input {
        match side {
            Side::Left => self.a,
            Side::Right => self.b,
        }
    }
}

/// Game state snapshot for synchronization
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Snapshot {
    pub tick: Tick,
    pub status: Status,
    pub paddles: [Paddle; 2],
    pub ball: Ball,
    pub score: [u8; 2],
    pub rng: u64,
}

/// Screen rectangle for pre-computed rendering coordinates
#[derive(Debug, Copy, Clone)]
pub struct ScreenRect {
    pub left: usize,
    pub right: usize,
    pub top: usize,
    pub bottom: usize,
}

impl ScreenRect {
    pub fn new(left: usize, right: usize, top: usize, bottom: usize) -> Self {
        ScreenRect {
            left,
            right,
            top,
            bottom,
        }
    }
}

/// Pure physics view - client agnostic game state
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub struct View {
    pub tick: Tick,
    pub status: Status,
    pub score: [u8; 2], // [left, right] scores

    // Pure physics data (no screen coordinates)
    pub left_paddle_y: Fx,
    pub right_paddle_y: Fx,
    pub paddle_half_h: Fx,
    pub ball_pos: Vec2,
    pub paddle_x_offset: Fx, // Distance from edge
    pub paddle_width: Fx,
    pub ball_radius: Fx,
}

/// Pixel-perfect rendering helper for consistent paddle heights
pub struct RenderHelper {
    field_width: usize,
    field_height: usize,
    paddle_height_pixels: usize, // Fixed height in pixels - calculated once
    paddle_width_pixels: usize,  // Fixed width in pixels - calculated once
}

impl RenderHelper {
    /// Create a new render helper with fixed paddle dimensions
    pub fn new(field_width: usize, field_height: usize, config: &Config) -> Self {
        // Calculate fixed paddle height in pixels (independent of position)
        let paddle_height_ratio = fx::to_f32(config.paddle_half_h) * 2.0; // Full height ratio
        let paddle_height_pixels = ((paddle_height_ratio * field_height as f32).max(2.0).round()
            as usize)
            .max(2) // Ensure minimum 2 pixels
            .min(field_height / 3); // Ensure reasonable maximum

        // Calculate fixed paddle width in pixels
        let paddle_width_ratio = fx::to_f32(config.paddle_width);
        let paddle_width_pixels = ((paddle_width_ratio * field_width as f32).max(1.0).round()
            as usize)
            .max(1) // Ensure minimum 1 pixel
            .min(field_width / 10); // Ensure reasonable maximum

        RenderHelper {
            field_width,
            field_height,
            paddle_height_pixels,
            paddle_width_pixels,
        }
    }

    /// Convert physics Y coordinate to screen Y coordinate
    pub fn physics_to_screen_y(&self, physics_y: Fx) -> usize {
        let clamped = fx::clamp_fx(physics_y, 0, FX_ONE);
        let normalized = fx::to_f32(clamped);
        // Y-axis inversion for screen coordinates
        let screen_coord = (1.0 - normalized) * (self.field_height - 1) as f32;
        (screen_coord + 0.5) as usize // Round to nearest pixel
    }

    /// Convert physics X coordinate to screen X coordinate
    pub fn physics_to_screen_x(&self, physics_x: Fx) -> usize {
        let clamped = fx::clamp_fx(physics_x, 0, FX_ONE);
        let normalized = fx::to_f32(clamped);
        let screen_coord = normalized * (self.field_width - 1) as f32;
        (screen_coord + 0.5) as usize // Round to nearest pixel
    }

    /// Get paddle rectangle with PERFECT consistent height - ALWAYS same height
    pub fn get_paddle_rect(&self, paddle_y: Fx, side: Side) -> ScreenRect {
        // Calculate paddle center in screen coordinates
        let center_y = self.physics_to_screen_y(paddle_y);

        // ABSOLUTELY guaranteed consistent height - never changes for any reason
        let half_height = self.paddle_height_pixels / 2;

        // Always use exact same top/bottom calculation
        // If this goes off-screen, so be it - consistency is more important
        let top = center_y.saturating_sub(half_height);
        let bottom = top + self.paddle_height_pixels - 1; // Always exactly paddle_height_pixels tall

        // Ensure we stay within bounds without changing height
        let (final_top, final_bottom) = if bottom >= self.field_height {
            // Slide the entire paddle up to fit, maintaining exact height
            let final_bottom = self.field_height - 1;
            let final_top = final_bottom - self.paddle_height_pixels + 1;
            (final_top, final_bottom)
        } else if top == 0 {
            // Already at top, height is correct
            (top, bottom)
        } else {
            // Normal case - paddle fits perfectly
            (top, bottom)
        };

        // Verify height is always consistent (debug assertion)
        debug_assert_eq!(
            final_bottom - final_top + 1,
            self.paddle_height_pixels,
            "Paddle height inconsistency! Expected {}, got {}",
            self.paddle_height_pixels,
            final_bottom - final_top + 1
        );

        // Calculate X position
        let paddle_x_physics = match side {
            Side::Left => fx::mul_fx(FX_ONE, fx::from_f32(0.05)), // 5% from left edge
            Side::Right => FX_ONE - fx::mul_fx(FX_ONE, fx::from_f32(0.05)), // 5% from right edge
        };

        let center_x = self.physics_to_screen_x(paddle_x_physics);
        let half_width = self.paddle_width_pixels / 2;
        let left = center_x.saturating_sub(half_width);
        let right = (center_x + half_width).min(self.field_width.saturating_sub(1));

        ScreenRect::new(left, right, final_top, final_bottom)
    }

    /// Get ball position in screen coordinates
    pub fn get_ball_position(&self, ball_pos: Vec2) -> (usize, usize) {
        (
            self.physics_to_screen_x(ball_pos.x),
            self.physics_to_screen_y(ball_pos.y),
        )
    }

    /// Get the fixed paddle height in pixels (always consistent)
    pub fn paddle_height_pixels(&self) -> usize {
        self.paddle_height_pixels
    }

    /// Get field dimensions
    pub fn field_dimensions(&self) -> (usize, usize) {
        (self.field_width, self.field_height)
    }
}

/// Game events that can occur during a tick
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub enum Event {
    Scored {
        scorer: Side,
        score: [u8; 2], // New score after this point
    },
}

/// Fixed-point utility functions
pub mod fx {
    use super::{Fx, FX_ONE};

    /// Convert from floating point
    pub fn from_f32(f: f32) -> Fx {
        (f * (FX_ONE as f32)) as Fx
    }

    /// Convert to floating point
    pub fn to_f32(value: Fx) -> f32 {
        (value as f32) / (FX_ONE as f32)
    }

    /// Multiply two fixed-point numbers
    pub fn mul_fx(a: Fx, b: Fx) -> Fx {
        ((a as i64) * (b as i64) >> 16) as Fx
    }

    /// Divide two fixed-point numbers
    pub fn div_fx(a: Fx, b: Fx) -> Fx {
        (((a as i64) << 16) / (b as i64)) as Fx
    }

    /// Absolute value
    pub fn abs_fx(a: Fx) -> Fx {
        a.abs()
    }

    /// Clamp between min and max
    pub fn clamp_fx(value: Fx, min: Fx, max: Fx) -> Fx {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_point_conversion() {
        assert_eq!(fx::from_f32(1.0), FX_ONE);
        assert_eq!(fx::from_f32(0.5), FX_ONE / 2);
        assert_eq!(fx::to_f32(FX_ONE), 1.0);
        assert_eq!(fx::to_f32(FX_ONE / 2), 0.5);
    }

    #[test]
    fn test_fixed_point_arithmetic() {
        let a = FX_ONE / 2; // 0.5
        let b = FX_ONE * 2; // 2.0

        assert_eq!(fx::mul_fx(a, b), FX_ONE); // 0.5 * 2.0 = 1.0
        assert_eq!(fx::div_fx(FX_ONE, a), b); // 1.0 / 0.5 = 2.0
    }

    #[test]
    fn test_side_opposite() {
        assert_eq!(Side::Left.opposite(), Side::Right);
        assert_eq!(Side::Right.opposite(), Side::Left);
    }

    #[test]
    fn test_input_ready() {
        let input_ready = Input::new(0, 1);
        let input_not_ready = Input::new(0, 0);

        assert!(input_ready.is_ready());
        assert!(!input_not_ready.is_ready());
    }

    #[test]
    fn test_input_pair_get_input() {
        let left_input = Input::new(127, 1);
        let right_input = Input::new(-127, 0);
        let pair = InputPair::new(0, left_input, right_input);

        assert_eq!(pair.get_input(Side::Left), left_input);
        assert_eq!(pair.get_input(Side::Right), right_input);
    }
}
