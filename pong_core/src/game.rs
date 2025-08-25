//! Main game logic and state management.

use crate::physics::Physics;
use crate::types::{fx, *};

/// Main game state and logic
pub struct Game {
    pub config: Config,
    pub tick: Tick,
    pub status: Status,
    pub paddles: [Paddle; 2],
    pub ball: Ball,
    pub score: [u8; 2],
    pub rng: u64,
}

impl Game {
    /// Create a new game with the given configuration
    pub fn new(config: Config) -> Self {
        let mut game = Game {
            config,
            tick: 0,
            status: Status::Lobby,
            paddles: [
                Paddle::new(FX_ONE / 2), // Left paddle at center
                Paddle::new(FX_ONE / 2), // Right paddle at center
            ],
            ball: Ball::new(Vec2::new(FX_ONE / 2, FX_ONE / 2), Vec2::zero()),
            score: [0, 0],
            rng: config.seed,
        };

        game.reset_for_serve(Side::Left);
        game
    }

    /// Step the game simulation forward by one tick
    pub fn step(&mut self, inputs: &InputPair) -> Option<Event> {
        debug_assert_eq!(inputs.tick, self.tick, "Input tick mismatch");

        let mut event = None;

        match self.status {
            Status::Lobby => {
                // Check if both players are ready
                if inputs.a.is_ready() && inputs.b.is_ready() {
                    self.status = Status::Countdown(180); // 3 seconds at 60 Hz
                }
            }

            Status::Countdown(ticks_remaining) => {
                if ticks_remaining <= 1 {
                    self.status = Status::Playing;
                    // Ball should already be set up from reset_for_serve
                } else {
                    self.status = Status::Countdown(ticks_remaining - 1);
                }
            }

            Status::Playing => {
                // Update paddles based on input
                Physics::update_paddle(&mut self.paddles[0], &inputs.a, &self.config);
                Physics::update_paddle(&mut self.paddles[1], &inputs.b, &self.config);

                // Update ball physics
                Physics::update_ball(&mut self.ball, &self.config);

                // Check paddle collisions
                Physics::check_paddle_collision(
                    &mut self.ball,
                    &self.paddles[0],
                    Side::Left,
                    &self.config,
                );
                Physics::check_paddle_collision(
                    &mut self.ball,
                    &self.paddles[1],
                    Side::Right,
                    &self.config,
                );

                // Limit ball speed to prevent runaway using proper fixed-point math
                let max_speed = fx::mul_fx(self.config.ball_speed, 4 * FX_ONE); // Allow 4x base speed max
                Physics::limit_ball_speed(&mut self.ball, max_speed);

                // Check for scoring
                if let Some(scorer) = Physics::check_scoring(&self.ball) {
                    self.handle_score(scorer);

                    event = Some(Event::Scored {
                        scorer,
                        score: self.score,
                    });
                }
            }

            Status::Scored(_, ticks_remaining) => {
                if ticks_remaining <= 1 {
                    // Check for game over
                    if self.score[0] >= self.config.max_score
                        || self.score[1] >= self.config.max_score
                    {
                        let winner = if self.score[0] >= self.config.max_score {
                            Side::Left
                        } else {
                            Side::Right
                        };
                        self.status = Status::GameOver(winner);
                    } else {
                        // Continue playing - serve to the side that was scored on
                        let server = if let Status::Scored(scorer, _) = self.status {
                            scorer.opposite() // Scored-on side serves next
                        } else {
                            Side::Left // Fallback
                        };

                        self.reset_for_serve(server);
                        self.status = Status::Playing;
                    }
                } else {
                    self.status = Status::Scored(
                        if let Status::Scored(side, _) = self.status {
                            side
                        } else {
                            Side::Left
                        },
                        ticks_remaining - 1,
                    );
                }
            }

            Status::GameOver(_) => {
                // Game is over, wait for restart or rematch
                // In M1, we don't handle this - would be handled by the client
            }
        }

        self.tick += 1;
        event
    }

    /// Generate a view of the current game state for rendering
    pub fn view(&self) -> View {
        View {
            tick: self.tick,
            status: self.status,
            left_y: self.paddles[0].y,
            right_y: self.paddles[1].y,
            paddle_half_h: self.config.paddle_half_h,
            ball_pos: self.ball.pos,
            score: self.score,
        }
    }

    /// Create a snapshot of the current game state
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            tick: self.tick,
            status: self.status,
            paddles: self.paddles,
            ball: self.ball,
            score: self.score,
            rng: self.rng,
        }
    }

    /// Restore game state from a snapshot
    pub fn restore(&mut self, snapshot: &Snapshot) {
        self.tick = snapshot.tick;
        self.status = snapshot.status;
        self.paddles = snapshot.paddles;
        self.ball = snapshot.ball;
        self.score = snapshot.score;
        self.rng = snapshot.rng;
    }

    /// Reset the game for a new match (rematch)
    pub fn reset_match(&mut self) {
        self.tick = 0;
        self.status = Status::Lobby;
        self.score = [0, 0];
        self.paddles[0].y = FX_ONE / 2;
        self.paddles[1].y = FX_ONE / 2;
        self.paddles[0].vy = 0;
        self.paddles[1].vy = 0;
        self.rng = self.config.seed;
        self.reset_for_serve(Side::Left);
    }

    /// Handle a scoring event
    fn handle_score(&mut self, scorer: Side) {
        match scorer {
            Side::Left => self.score[0] += 1,
            Side::Right => self.score[1] += 1,
        }

        self.status = Status::Scored(scorer, 180); // 3 seconds pause
    }

    /// Reset ball and game state for a serve
    fn reset_for_serve(&mut self, serving_side: Side) {
        Physics::serve_ball(&mut self.ball, serving_side, &self.config, &mut self.rng);
    }

    /// Get the current winner (if game is over)
    pub fn winner(&self) -> Option<Side> {
        match self.status {
            Status::GameOver(winner) => Some(winner),
            _ => None,
        }
    }

    /// Check if the game is active (accepting inputs)
    pub fn is_active(&self) -> bool {
        matches!(self.status, Status::Playing)
    }

    /// Get a human-readable status string
    pub fn status_string(&self) -> &'static str {
        match self.status {
            Status::Lobby => "Waiting for players",
            Status::Countdown(_) => "Get ready...",
            Status::Playing => "Playing",
            Status::Scored(_, _) => "Point scored!",
            Status::GameOver(_) => "Game over",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let config = Config::default();
        let game = Game::new(config);

        assert_eq!(game.tick, 0);
        assert_eq!(game.status, Status::Lobby);
        assert_eq!(game.score, [0, 0]);
        assert_eq!(game.ball.pos.x, FX_ONE / 2);
        assert_eq!(game.ball.pos.y, FX_ONE / 2);
    }

    #[test]
    fn test_lobby_to_countdown() {
        let mut game = Game::new(Config::default());

        // Both players ready
        let inputs = InputPair::new(0, Input::new(0, 1), Input::new(0, 1));
        let event = game.step(&inputs);

        assert!(matches!(game.status, Status::Countdown(_)));
        assert!(event.is_none());
    }

    #[test]
    fn test_countdown_to_playing() {
        let mut game = Game::new(Config::default());
        game.status = Status::Countdown(1);

        let inputs = InputPair::new(0, Input::zero(), Input::zero());
        let event = game.step(&inputs);

        assert_eq!(game.status, Status::Playing);
        assert!(event.is_none());
    }

    #[test]
    fn test_paddle_movement_during_play() {
        let mut game = Game::new(Config::default());
        game.status = Status::Playing;

        let initial_left_y = game.paddles[0].y;

        // Move left paddle up
        let inputs = InputPair::new(0, Input::new(127, 0), Input::zero());
        game.step(&inputs);

        assert_ne!(game.paddles[0].y, initial_left_y);
        assert!(game.paddles[0].vy > 0); // Moving up
    }

    #[test]
    fn test_scoring() {
        let mut game = Game::new(Config::default());
        game.status = Status::Playing;

        // Move ball past right edge to simulate scoring
        game.ball.pos.x = FX_ONE + 1000;

        let inputs = InputPair::new(0, Input::zero(), Input::zero());
        let event = game.step(&inputs);

        // Left player should have scored
        assert_eq!(game.score[0], 1);
        assert!(matches!(game.status, Status::Scored(Side::Left, _)));

        if let Some(Event::Scored { scorer, score }) = event {
            assert_eq!(scorer, Side::Left);
            assert_eq!(score, [1, 0]);
        } else {
            panic!("Expected scoring event");
        }
    }

    #[test]
    fn test_game_over() {
        let mut game = Game::new(Config::default());
        game.score[0] = game.config.max_score; // Left player at max score
        game.status = Status::Scored(Side::Left, 1); // About to transition

        let inputs = InputPair::new(0, Input::zero(), Input::zero());
        game.step(&inputs);

        assert!(matches!(game.status, Status::GameOver(Side::Left)));
        assert_eq!(game.winner(), Some(Side::Left));
    }

    #[test]
    fn test_serve_after_score() {
        let mut game = Game::new(Config::default());
        game.score = [1, 0];
        game.status = Status::Scored(Side::Left, 1); // Left scored, about to serve

        let inputs = InputPair::new(0, Input::zero(), Input::zero());
        game.step(&inputs);

        // Should be playing again
        assert_eq!(game.status, Status::Playing);
        // Ball should be at center
        assert_eq!(game.ball.pos.x, FX_ONE / 2);
        assert_eq!(game.ball.pos.y, FX_ONE / 2);
        // Ball should be moving (right side serves since left scored)
        assert!(game.ball.vel.x != 0 || game.ball.vel.y != 0);
    }

    #[test]
    fn test_snapshot_and_restore() {
        let mut game1 = Game::new(Config::default());
        game1.tick = 100;
        game1.score[0] = 3;
        game1.score[1] = 2;
        game1.status = Status::Playing;

        let snapshot = game1.snapshot();

        let mut game2 = Game::new(Config::default());
        game2.restore(&snapshot);

        assert_eq!(game1.tick, game2.tick);
        assert_eq!(game1.status, game2.status);
        assert_eq!(game1.score, game2.score);
        assert_eq!(game1.paddles[0].y, game2.paddles[0].y);
        assert_eq!(game1.paddles[1].y, game2.paddles[1].y);
        assert_eq!(game1.ball.pos, game2.ball.pos);
        assert_eq!(game1.ball.vel, game2.ball.vel);
        assert_eq!(game1.rng, game2.rng);
    }

    #[test]
    fn test_reset_match() {
        let mut game = Game::new(Config::default());
        game.tick = 1000;
        game.score = [5, 3];
        game.status = Status::GameOver(Side::Left);

        game.reset_match();

        assert_eq!(game.tick, 0);
        assert_eq!(game.status, Status::Lobby);
        assert_eq!(game.score, [0, 0]);
        assert_eq!(game.paddles[0].y, FX_ONE / 2);
        assert_eq!(game.paddles[1].y, FX_ONE / 2);
    }

    #[test]
    fn test_view_generation() {
        let game = Game::new(Config::default());
        let view = game.view();

        assert_eq!(view.tick, game.tick);
        assert_eq!(view.status, game.status);
        assert_eq!(view.left_y, game.paddles[0].y);
        assert_eq!(view.right_y, game.paddles[1].y);
        assert_eq!(view.paddle_half_h, game.config.paddle_half_h);
        assert_eq!(view.ball_pos, game.ball.pos);
        assert_eq!(view.score, game.score);
    }

    #[test]
    fn test_deterministic_simulation() {
        let config = Config::default();
        let mut game1 = Game::new(config);
        let mut game2 = Game::new(config);

        // Apply same inputs to both games
        let test_inputs = [
            InputPair::new(0, Input::new(0, 1), Input::new(0, 1)), // Ready
            InputPair::new(1, Input::new(50, 0), Input::new(-30, 0)), // Play
            InputPair::new(2, Input::new(-20, 0), Input::new(75, 0)), // Play
        ];

        for inputs in &test_inputs {
            game1.step(inputs);
            game2.step(inputs);
        }

        // Games should be in identical states
        assert_eq!(game1.tick, game2.tick);
        assert_eq!(game1.status, game2.status);
        assert_eq!(game1.paddles, game2.paddles);
        assert_eq!(game1.ball.pos, game2.ball.pos);
        assert_eq!(game1.ball.vel, game2.ball.vel);
        assert_eq!(game1.score, game2.score);
        assert_eq!(game1.rng, game2.rng);
    }

    #[test]
    fn test_is_active() {
        let mut game = Game::new(Config::default());

        assert!(!game.is_active()); // Lobby

        game.status = Status::Playing;
        assert!(game.is_active());

        game.status = Status::GameOver(Side::Left);
        assert!(!game.is_active());
    }

    #[test]
    fn test_status_string() {
        let mut game = Game::new(Config::default());

        assert_eq!(game.status_string(), "Waiting for players");

        game.status = Status::Playing;
        assert_eq!(game.status_string(), "Playing");

        game.status = Status::GameOver(Side::Left);
        assert_eq!(game.status_string(), "Game over");
    }
}
