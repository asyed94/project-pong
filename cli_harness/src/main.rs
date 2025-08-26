//! CLI harness for testing pong_core with two local players.

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement, Clear, ClearType,
        EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use pong_core::{fx, *};
use std::io::{stdout, Result, Write};
use std::time::{Duration, Instant};

/// CLI application state
struct CliApp {
    game: Game,
    running: bool,
    last_tick: Instant,
    input_system: InputSystem,
    show_help: bool,
}

/// Keyboard capability detection
#[derive(Debug, Clone, Copy)]
enum KeyboardMode {
    Enhanced, // Supports KeyEventKind::Release
    Momentum, // Fallback momentum-based
}

fn detect_keyboard_capabilities() -> KeyboardMode {
    // Use crossterm's built-in detection
    match supports_keyboard_enhancement() {
        Ok(true) => KeyboardMode::Enhanced,
        Ok(false) | Err(_) => KeyboardMode::Momentum,
    }
}

/// Enhanced input system (direct key hold detection)
#[derive(Default)]
struct EnhancedInput {
    up_held: bool,
    down_held: bool,
    ready_held: bool,
}

impl EnhancedInput {
    fn handle_key_event(&mut self, event: KeyEvent) {
        match (event.code, event.kind) {
            (KeyCode::Char('w') | KeyCode::Char('W'), KeyEventKind::Press) => self.up_held = true,
            (KeyCode::Char('w') | KeyCode::Char('W'), KeyEventKind::Release) => {
                self.up_held = false
            }
            (KeyCode::Char('s') | KeyCode::Char('S'), KeyEventKind::Press) => self.down_held = true,
            (KeyCode::Char('s') | KeyCode::Char('S'), KeyEventKind::Release) => {
                self.down_held = false
            }
            (KeyCode::Char(' '), KeyEventKind::Press) => self.ready_held = true,
            (KeyCode::Char(' '), KeyEventKind::Release) => self.ready_held = false,
            _ => {}
        }
    }

    fn to_game_input(&self) -> Input {
        let axis_y = if self.up_held && !self.down_held {
            -127 // UP
        } else if self.down_held && !self.up_held {
            127 // DOWN
        } else {
            0 // Stop
        };

        let buttons = if self.ready_held { 1 } else { 0 };
        Input::new(axis_y, buttons)
    }
}

/// Momentum-based input system (keypress accumulation)
#[derive(Default)]
struct MomentumInput {
    momentum: f32,
    ready: bool,
}

impl MomentumInput {
    fn handle_keypress(&mut self, code: KeyCode) {
        let input_direction = match code {
            KeyCode::Char('w') | KeyCode::Char('W') => -1.0, // UP
            KeyCode::Char('s') | KeyCode::Char('S') => 1.0,  // DOWN
            KeyCode::Char(' ') => {
                self.ready = true;
                return;
            }
            _ => return,
        };

        let current_direction = self.momentum.signum();

        if current_direction != 0.0 && input_direction != current_direction {
            // OPPOSITE DIRECTION: Apply strong braking for quick stops
            let brake_strength = 0.75; // 75% momentum reduction per opposite tap
            self.momentum *= (1.0 - brake_strength);

            // Add small amount in new direction for immediate response
            self.momentum += input_direction * 0.15;
        } else {
            // SAME DIRECTION: Use progressive acceleration for quick ramp-up
            let current_speed = self.momentum.abs();
            let momentum_gain = if current_speed < 0.3 {
                0.4 // Quick initial acceleration (0 → 30% in 1 tap)
            } else if current_speed < 0.7 {
                0.35 // Fast mid-range acceleration
            } else {
                0.25 // Slower approach to max (requires more skill)
            };

            self.momentum += input_direction * momentum_gain;
        }

        // Clamp to limits
        self.momentum = self.momentum.clamp(-1.0, 1.0);
    }

    fn update_momentum(&mut self) {
        // Variable friction based on speed for natural feel
        let friction = match self.momentum.abs() {
            speed if speed > 0.9 => 0.90, // High speed: strong friction
            speed if speed > 0.5 => 0.94, // Medium speed: normal friction
            speed if speed > 0.1 => 0.97, // Low speed: gentle friction
            _ => 1.0,                     // Dead zone: no friction
        };

        self.momentum *= friction;

        // Enhanced stopping threshold
        if self.momentum.abs() < 0.02 {
            self.momentum = 0.0;
        }
    }

    fn to_game_input(&self) -> Input {
        let axis_y = (self.momentum * 127.0) as i8;
        let buttons = if self.ready { 1 } else { 0 };
        Input::new(axis_y, buttons)
    }
}

/// Unified input system that adapts to terminal capabilities
enum InputSystem {
    Enhanced {
        p1: EnhancedInput,
        p2: EnhancedInput,
        mode: KeyboardMode,
    },
    Momentum {
        p1: MomentumInput,
        p2: MomentumInput,
        mode: KeyboardMode,
    },
}

impl InputSystem {
    fn new() -> Self {
        match detect_keyboard_capabilities() {
            KeyboardMode::Enhanced => InputSystem::Enhanced {
                p1: EnhancedInput::default(),
                p2: EnhancedInput::default(),
                mode: KeyboardMode::Enhanced,
            },
            KeyboardMode::Momentum => InputSystem::Momentum {
                p1: MomentumInput::default(),
                p2: MomentumInput::default(),
                mode: KeyboardMode::Momentum,
            },
        }
    }

    fn get_mode_description(&self) -> &'static str {
        match self {
            InputSystem::Enhanced { .. } => "Enhanced (Hold keys)",
            InputSystem::Momentum { .. } => "Momentum (Tap keys)",
        }
    }

    fn get_inputs(&self) -> (Input, Input) {
        match self {
            InputSystem::Enhanced { p1, p2, .. } => (p1.to_game_input(), p2.to_game_input()),
            InputSystem::Momentum { p1, p2, .. } => (p1.to_game_input(), p2.to_game_input()),
        }
    }

    fn reset(&mut self) {
        match self {
            InputSystem::Enhanced { p1, p2, .. } => {
                *p1 = EnhancedInput::default();
                *p2 = EnhancedInput::default();
            }
            InputSystem::Momentum { p1, p2, .. } => {
                *p1 = MomentumInput::default();
                *p2 = MomentumInput::default();
            }
        }
    }
}

impl CliApp {
    fn new() -> Self {
        let config = Config::default();
        let input_system = InputSystem::new();

        // Display which input mode was detected
        eprintln!("🎮 Input mode: {}", input_system.get_mode_description());

        Self {
            game: Game::new(config),
            running: true,
            last_tick: Instant::now(),
            input_system,
            show_help: true,
        }
    }

    fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;

        let mut stdout = stdout();
        let supports_enhancement = matches!(supports_keyboard_enhancement(), Ok(true));

        // Enable keyboard enhancement if supported
        if supports_enhancement && matches!(self.input_system, InputSystem::Enhanced { .. }) {
            queue!(
                stdout,
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                        | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                )
            )?;
        }

        execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

        while self.running {
            self.handle_input()?;
            self.update()?;
            self.render()?;

            // Target 60 FPS
            let frame_time = Duration::from_millis(1000 / 60);
            std::thread::sleep(frame_time.saturating_sub(self.last_tick.elapsed()));
        }

        // Cleanup keyboard enhancement
        if supports_enhancement && matches!(self.input_system, InputSystem::Enhanced { .. }) {
            queue!(stdout, PopKeyboardEnhancementFlags)?;
        }

        execute!(stdout, LeaveAlternateScreen, Show)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<()> {
        // Process all available key events
        while poll(Duration::from_millis(0))? {
            if let Event::Key(event) = read()? {
                // Handle system keys (quit, help, reset)
                if event.kind == KeyEventKind::Press {
                    match event.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            self.running = false;
                        }
                        KeyCode::Char('h') | KeyCode::Char('H') => {
                            self.show_help = !self.show_help;
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            self.game.reset_match();
                            self.input_system.reset();
                        }
                        KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.running = false;
                        }
                        _ => {}
                    }
                }

                // Handle movement/game inputs based on mode
                match &mut self.input_system {
                    InputSystem::Enhanced { p1, p2, .. } => {
                        // Enhanced mode: Handle both press and release events
                        match event.code {
                            KeyCode::Char('w')
                            | KeyCode::Char('W')
                            | KeyCode::Char('s')
                            | KeyCode::Char('S') => {
                                p1.handle_key_event(event);
                            }
                            KeyCode::Up | KeyCode::Down => {
                                // Map arrow keys to Player 2 using WASD equivalents
                                let mapped_event = KeyEvent {
                                    code: match event.code {
                                        KeyCode::Up => KeyCode::Char('w'),
                                        KeyCode::Down => KeyCode::Char('s'),
                                        _ => event.code,
                                    },
                                    modifiers: event.modifiers,
                                    kind: event.kind,
                                    state: event.state,
                                };
                                p2.handle_key_event(mapped_event);
                            }
                            KeyCode::Char(' ') => {
                                p1.handle_key_event(event);
                                p2.handle_key_event(event);
                            }
                            _ => {}
                        }
                    }
                    InputSystem::Momentum { p1, p2, .. } => {
                        // Momentum mode: Only handle keypress events
                        if event.kind == KeyEventKind::Press {
                            match event.code {
                                KeyCode::Char('w')
                                | KeyCode::Char('W')
                                | KeyCode::Char('s')
                                | KeyCode::Char('S') => {
                                    p1.handle_keypress(event.code);
                                }
                                KeyCode::Up => {
                                    p2.handle_keypress(KeyCode::Char('w'));
                                }
                                KeyCode::Down => {
                                    p2.handle_keypress(KeyCode::Char('s'));
                                }
                                KeyCode::Char(' ') => {
                                    p1.handle_keypress(event.code);
                                    p2.handle_keypress(event.code);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Update momentum for momentum-based input
        if let InputSystem::Momentum { p1, p2, .. } = &mut self.input_system {
            p1.update_momentum();
            p2.update_momentum();
        }

        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick);

        // Target 60 Hz tick rate
        if dt >= Duration::from_millis(1000 / 60) {
            let old_status = self.game.status;
            let view = self.game.view();
            let (p1_input, p2_input) = self.input_system.get_inputs();
            let inputs = InputPair::new(view.tick, p1_input, p2_input);

            if let Some(event) = self.game.step(&inputs) {
                match event {
                    pong_core::Event::Scored { scorer, score } => {
                        // Could add sound or visual feedback here
                        let _ = scorer; // Suppress unused warning
                        let _ = score;
                    }
                }
            }

            // Reset ready flags only after successful transition to countdown
            if matches!(old_status, Status::Lobby)
                && matches!(self.game.status, Status::Countdown(_))
            {
                // Reset ready state in input system
                match &mut self.input_system {
                    InputSystem::Enhanced { p1, p2, .. } => {
                        p1.ready_held = false;
                        p2.ready_held = false;
                    }
                    InputSystem::Momentum { p1, p2, .. } => {
                        p1.ready = false;
                        p2.ready = false;
                    }
                }
            }

            self.last_tick = now;
        }

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        let view = self.game.view();
        let mut row = 0;

        execute!(stdout(), Clear(ClearType::All))?;

        // Header
        execute!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::Cyan),
            Print("🏓 PONG CLI HARNESS 🏓"),
            ResetColor
        )?;
        row += 1;

        // Game status
        execute!(
            stdout(),
            MoveTo(0, row),
            Print(format!("Status: {}", self.game.status_string()))
        )?;
        row += 1;

        // Score
        execute!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::Yellow),
            Print(format!("Score: {} - {}", view.score[0], view.score[1])),
            ResetColor
        )?;
        row += 1;

        // Game field
        row = self.render_field(&view, row)?;

        // Help
        if self.show_help {
            self.render_help(row)?;
        }

        stdout().flush()?;
        Ok(())
    }

    fn render_field(&self, view: &View, mut row: u16) -> Result<u16> {
        const FIELD_WIDTH: usize = 60;
        const FIELD_HEIGHT: usize = 20;

        // Convert normalized coordinates to screen coordinates
        let ball_x =
            ((fx::to_f32(view.ball_pos.x).clamp(0.0, 1.0)) * (FIELD_WIDTH - 1) as f32) as usize;
        let ball_y =
            ((fx::to_f32(view.ball_pos.y).clamp(0.0, 1.0)) * (FIELD_HEIGHT - 1) as f32) as usize;

        let left_paddle_y =
            ((fx::to_f32(view.left_paddle_y).clamp(0.0, 1.0)) * (FIELD_HEIGHT - 1) as f32) as usize;
        let right_paddle_y = ((fx::to_f32(view.right_paddle_y).clamp(0.0, 1.0))
            * (FIELD_HEIGHT - 1) as f32) as usize;

        let paddle_height = ((fx::to_f32(view.paddle_half_h) * 2.0) * FIELD_HEIGHT as f32) as usize;
        let paddle_half_h = paddle_height / 2;

        // Render top border
        execute!(stdout(), MoveTo(0, row), Print("┌"))?;
        for _ in 0..FIELD_WIDTH {
            execute!(stdout(), Print("─"))?;
        }
        execute!(stdout(), Print("┐"))?;
        row += 1;

        // Render field content
        for y in 0..FIELD_HEIGHT {
            execute!(stdout(), MoveTo(0, row), Print("│"))?;

            for x in 0..FIELD_WIDTH {
                let mut char_to_print = ' ';
                let mut color = Color::White;

                // Ball
                if x == ball_x && y == ball_y {
                    char_to_print = '●';
                    color = Color::Red;
                }
                // Left paddle (x = 0-2)
                else if x <= 2
                    && y >= left_paddle_y.saturating_sub(paddle_half_h)
                    && y <= (left_paddle_y + paddle_half_h).min(FIELD_HEIGHT - 1)
                {
                    char_to_print = '█';
                    color = Color::Blue;
                }
                // Right paddle (x = FIELD_WIDTH-3 to FIELD_WIDTH-1)
                else if x >= FIELD_WIDTH.saturating_sub(3)
                    && y >= right_paddle_y.saturating_sub(paddle_half_h)
                    && y <= (right_paddle_y + paddle_half_h).min(FIELD_HEIGHT - 1)
                {
                    char_to_print = '█';
                    color = Color::Green;
                }
                // Center line
                else if x == FIELD_WIDTH / 2 {
                    char_to_print = '┊';
                    color = Color::DarkGrey;
                }

                execute!(
                    stdout(),
                    SetForegroundColor(color),
                    Print(char_to_print),
                    ResetColor
                )?;
            }

            execute!(stdout(), Print("│"))?;
            row += 1;
        }

        // Render bottom border
        execute!(stdout(), MoveTo(0, row), Print("└"))?;
        for _ in 0..FIELD_WIDTH {
            execute!(stdout(), Print("─"))?;
        }
        execute!(stdout(), Print("┘"))?;
        row += 1;

        Ok(row)
    }

    fn render_help(&self, mut row: u16) -> Result<()> {
        // Empty line
        row += 1;

        // Controls header
        execute!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::DarkGrey),
            Print("--- CONTROLS ---"),
            ResetColor
        )?;
        row += 1;

        // Player 1 controls
        execute!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::DarkGrey),
            Print("Player 1 (Blue):  W/S to move up/down"),
            ResetColor
        )?;
        row += 1;

        // Player 2 controls
        execute!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::DarkGrey),
            Print("Player 2 (Green): ↑/↓ to move up/down"),
            ResetColor
        )?;
        row += 1;

        // Action controls
        execute!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::DarkGrey),
            Print("SPACE: Ready/Serve  |  R: Reset  |  H: Toggle help  |  Q: Quit"),
            ResetColor
        )?;
        row += 1;

        // Input mode and debug info
        execute!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "Input Mode: {} | Current inputs: P1({:3}) P2({:3})",
                self.input_system.get_mode_description(),
                self.input_system.get_inputs().0.axis_y,
                self.input_system.get_inputs().1.axis_y
            )),
            ResetColor
        )?;
        row += 1;

        // Game state debug info
        let view = self.game.view();
        execute!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "Tick: {} | Status: {:?} | Ball: ({:.2}, {:.2})",
                view.tick,
                view.status,
                fx::to_f32(view.ball_pos.x),
                fx::to_f32(view.ball_pos.y)
            )),
            ResetColor
        )?;
        row += 1;

        // Ball velocity debug (if we can access it)
        execute!(
            stdout(),
            MoveTo(0, row),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "Ball vel: ({:.2}, {:.2}) | Paddle Y: L={:.2} R={:.2}",
                fx::to_f32(self.game.ball.vel.x),
                fx::to_f32(self.game.ball.vel.y),
                fx::to_f32(view.left_paddle_y),
                fx::to_f32(view.right_paddle_y)
            )),
            ResetColor
        )?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut app = CliApp::new();

    // Handle Ctrl+C gracefully
    ctrlc::set_handler(move || {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), Show);
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    app.run()
}
