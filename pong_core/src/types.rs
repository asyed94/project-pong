//! Core types and constants for the deterministic Pong game engine.

/// Fixed-point type using 16.16 format (16 integer bits, 16 fractional bits)
pub type Fx = i32;

/// One unit in fixed-point format
pub const FX_ONE: Fx = 1 << 16;

/// Tick counter type
pub type Tick = u32;

/// Player/paddle side
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
}

impl Default for Config {
    fn default() -> Self {
        Config {
            paddle_half_h: FX_ONE / 8,             // 1/8 unit
            paddle_speed: (FX_ONE * 3) / 2,        // 1.5 units/s
            ball_speed: FX_ONE / 2,                // 0.5 units/s
            ball_speed_up: FX_ONE + (FX_ONE / 20), // +5% per hit
            wall_thickness: 0,
            paddle_x: FX_ONE / 20, // 5% from edge
            max_score: 11,
            seed: 0xC0FFEE,
            tick_hz: 60,
        }
    }
}

/// 2D vector in fixed-point
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// View data for rendering (normalized coordinates)
#[derive(Debug, Copy, Clone)]
pub struct View {
    pub tick: Tick,
    pub status: Status,
    pub left_y: Fx,        // Left paddle center Y
    pub right_y: Fx,       // Right paddle center Y
    pub paddle_half_h: Fx, // Half-height of paddles
    pub ball_pos: Vec2,    // Ball position
    pub score: [u8; 2],    // [left, right] scores
}

/// Game events that can occur during a tick
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
