use crate::event::{AppEvent, Event, EventHandler};
use pong_core::{Config, Game, Input, InputPair, Status};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    DefaultTerminal,
};
use std::time::Instant;

/// Game key mapping for cleaner input handling
#[derive(Debug, Clone, Copy)]
enum GameKey {
    Player1Up,
    Player1Down,
    Player2Up,
    Player2Down,
    Ready,
}

fn map_keycode_to_game_key(code: KeyCode) -> Option<GameKey> {
    match code {
        KeyCode::Char('w') | KeyCode::Char('W') => Some(GameKey::Player1Up),
        KeyCode::Char('s') | KeyCode::Char('S') => Some(GameKey::Player1Down),
        KeyCode::Up => Some(GameKey::Player2Up),
        KeyCode::Down => Some(GameKey::Player2Down),
        KeyCode::Char(' ') => Some(GameKey::Ready),
        _ => None,
    }
}

/// Common trait for player input handling
trait PlayerInput {
    fn handle_up(&mut self, pressed: bool);
    fn handle_down(&mut self, pressed: bool);
    fn handle_ready(&mut self, pressed: bool);
    fn to_game_input(&self) -> Input;
    fn reset(&mut self);
    fn update(&mut self) {} // Default no-op, overridden by momentum
}

/// Enhanced input implementation
#[derive(Default)]
struct EnhancedPlayerInput {
    up_held: bool,
    down_held: bool,
    ready_held: bool,
}

impl PlayerInput for EnhancedPlayerInput {
    fn handle_up(&mut self, pressed: bool) {
        self.up_held = pressed;
    }
    fn handle_down(&mut self, pressed: bool) {
        self.down_held = pressed;
    }
    fn handle_ready(&mut self, pressed: bool) {
        self.ready_held = pressed;
    }

    fn to_game_input(&self) -> Input {
        let axis_y = if self.up_held && !self.down_held {
            127 // UP = positive axis_y
        } else if self.down_held && !self.up_held {
            -127 // DOWN = negative axis_y
        } else {
            0 // Stop
        };
        Input::new(axis_y, if self.ready_held { 1 } else { 0 })
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Momentum input implementation
#[derive(Default)]
struct MomentumPlayerInput {
    momentum: f32,
    ready: bool,
}

impl PlayerInput for MomentumPlayerInput {
    fn handle_up(&mut self, pressed: bool) {
        if pressed {
            self.apply_direction(1.0); // UP = positive direction
        }
    }

    fn handle_down(&mut self, pressed: bool) {
        if pressed {
            self.apply_direction(-1.0); // DOWN = negative direction
        }
    }

    fn handle_ready(&mut self, pressed: bool) {
        if pressed {
            self.ready = true;
        }
    }

    fn to_game_input(&self) -> Input {
        Input::new(
            (self.momentum * 127.0) as i8,
            if self.ready { 1 } else { 0 },
        )
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn update(&mut self) {
        let friction = match self.momentum.abs() {
            speed if speed > 0.9 => 0.90,
            speed if speed > 0.5 => 0.94,
            speed if speed > 0.1 => 0.97,
            _ => 1.0,
        };
        self.momentum *= friction;
        if self.momentum.abs() < 0.02 {
            self.momentum = 0.0;
        }
    }
}

impl MomentumPlayerInput {
    fn apply_direction(&mut self, direction: f32) {
        let current_direction = self.momentum.signum();

        if current_direction != 0.0 && direction != current_direction {
            // Opposite direction: brake and change
            self.momentum *= 0.25; // Strong braking
            self.momentum += direction * 0.15;
        } else {
            // Same direction: accelerate
            let gain = match self.momentum.abs() {
                speed if speed < 0.3 => 0.4,
                speed if speed < 0.7 => 0.35,
                _ => 0.25,
            };
            self.momentum += direction * gain;
        }
        self.momentum = self.momentum.clamp(-1.0, 1.0);
    }
}

/// Simplified input system with adaptive capabilities
struct InputSystem {
    p1: Box<dyn PlayerInput>,
    p2: Box<dyn PlayerInput>,
    is_enhanced: bool,
}

impl InputSystem {
    fn new() -> Self {
        let is_enhanced = matches!(
            crossterm::terminal::supports_keyboard_enhancement(),
            Ok(true)
        );

        if is_enhanced {
            Self {
                p1: Box::new(EnhancedPlayerInput::default()),
                p2: Box::new(EnhancedPlayerInput::default()),
                is_enhanced: true,
            }
        } else {
            Self {
                p1: Box::new(MomentumPlayerInput::default()),
                p2: Box::new(MomentumPlayerInput::default()),
                is_enhanced: false,
            }
        }
    }

    fn get_mode_description(&self) -> &'static str {
        if self.is_enhanced {
            "Enhanced (Hold keys)"
        } else {
            "Momentum (Tap keys)"
        }
    }

    fn get_inputs(&self) -> (Input, Input) {
        (self.p1.to_game_input(), self.p2.to_game_input())
    }

    fn reset(&mut self) {
        self.p1.reset();
        self.p2.reset();
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        if let Some(game_key) = map_keycode_to_game_key(event.code) {
            let pressed = if self.is_enhanced {
                event.kind == KeyEventKind::Press
            } else {
                event.kind == KeyEventKind::Press
            };

            let released = event.kind == KeyEventKind::Release;

            match game_key {
                GameKey::Player1Up => self.p1.handle_up(pressed && !released),
                GameKey::Player1Down => self.p1.handle_down(pressed && !released),
                GameKey::Player2Up => self.p2.handle_up(pressed && !released),
                GameKey::Player2Down => self.p2.handle_down(pressed && !released),
                GameKey::Ready => {
                    self.p1.handle_ready(pressed && !released);
                    self.p2.handle_ready(pressed && !released);
                }
            }
        }
    }

    fn update_momentum(&mut self) {
        self.p1.update();
        self.p2.update();
    }

    fn reset_ready_after_countdown(&mut self, old_status: Status, new_status: Status) {
        if matches!(old_status, Status::Lobby) && matches!(new_status, Status::Countdown(_)) {
            self.p1.handle_ready(false);
            self.p2.handle_ready(false);
        }
    }
}

/// Application screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppScreen {
    Start,
    Local,
    Game,
}

/// Local game modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalMode {
    VsLocal2,
}

/// Menu states for navigation
#[derive(Debug, Default)]
pub struct MenuState {
    pub start_selected: usize,
    pub local_selected: usize,
}

/// Game board size constants
const MIN_GAME_WIDTH: u16 = 60;
const MIN_GAME_HEIGHT: u16 = 20;
const FIXED_GAME_WIDTH: usize = 80;
const FIXED_GAME_HEIGHT: usize = 24;

/// Main application
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Current screen
    pub screen: AppScreen,
    /// Menu states
    pub menu_state: MenuState,
    /// Local game mode
    pub local_mode: LocalMode,
    /// Game instance
    pub game: Option<Game>,
    /// Input system (cli_harness style)
    pub input_system: InputSystem,
    /// Last game tick
    pub last_tick: Instant,
    /// Event handler
    pub events: EventHandler,
    /// Current terminal size
    pub terminal_size: (u16, u16),
    /// Whether UI is paused due to small terminal
    pub ui_paused: bool,
}

impl App {
    /// Constructs a new instance of App
    pub fn new() -> color_eyre::Result<Self> {
        let events = EventHandler::new()?;
        let input_system = InputSystem::new();

        // Display which input mode was detected
        eprintln!("🎮 Input mode: {}", input_system.get_mode_description());

        Ok(Self {
            running: true,
            screen: AppScreen::Start,
            menu_state: MenuState::default(),
            local_mode: LocalMode::VsLocal2,
            game: None,
            input_system,
            last_tick: Instant::now(),
            events,
            terminal_size: (80, 24), // Default size
            ui_paused: false,
        })
    }

    /// Run the application's main loop
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                ratatui::crossterm::event::Event::Key(key_event) => {
                    self.handle_key_event(key_event)?
                }
                _ => {}
            },
            Event::App(app_event) => self.handle_app_event(app_event),
        }
        Ok(())
    }

    /// Handle key events and convert to app events
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        // Global quit keys
        match key_event.code {
            KeyCode::Char('q') => {
                self.events.send(AppEvent::Quit);
                return Ok(());
            }
            KeyCode::Char('c') | KeyCode::Char('C')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                self.events.send(AppEvent::Quit);
                return Ok(());
            }
            _ => {}
        }

        // Screen-specific key handling
        match self.screen {
            AppScreen::Start | AppScreen::Local => {
                // Menu navigation - only on key press
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Up => self.events.send(AppEvent::MenuUp),
                        KeyCode::Down => self.events.send(AppEvent::MenuDown),
                        KeyCode::Enter => self.events.send(AppEvent::MenuSelect),
                        KeyCode::Esc => {
                            if self.screen == AppScreen::Local {
                                self.events.send(AppEvent::NavigateToStart);
                            } else {
                                self.events.send(AppEvent::Quit);
                            }
                        }
                        _ => {}
                    }
                }
            }
            AppScreen::Game => {
                // Game controls - use InputSystem directly like cli_harness
                match key_event.code {
                    KeyCode::Esc => {
                        if key_event.kind == KeyEventKind::Press {
                            self.events.send(AppEvent::NavigateToStart);
                        }
                    }
                    _ => {
                        // Let InputSystem handle all game input
                        self.input_system.handle_key_event(key_event);
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle application events
    fn handle_app_event(&mut self, app_event: AppEvent) {
        match app_event {
            AppEvent::Quit => self.quit(),
            AppEvent::NavigateToStart => self.navigate_to_start(),
            AppEvent::NavigateToLocal => self.navigate_to_local(),
            AppEvent::NavigateToGame => self.navigate_to_game(),
            AppEvent::MenuUp => self.menu_up(),
            AppEvent::MenuDown => self.menu_down(),
            AppEvent::MenuSelect => self.menu_select(),
            AppEvent::TerminalResize(width, height) => self.handle_resize(width, height),
        }
    }

    /// Handle tick event for game updates
    fn tick(&mut self) {
        // Update momentum for momentum-based input
        self.input_system.update_momentum();

        if let Some(ref mut game) = self.game {
            let now = Instant::now();
            let dt = now.duration_since(self.last_tick);

            // Update game at 60 FPS
            if dt >= std::time::Duration::from_millis(1000 / 60) {
                let old_status = game.status;
                let view = game.view();
                let (p1_input, p2_input) = self.input_system.get_inputs();
                let inputs = InputPair::new(view.tick, p1_input, p2_input);

                if let Some(_event) = game.step(&inputs) {
                    // Handle game events (scoring, etc.)
                }

                // Reset ready flags after successful transition to countdown
                self.input_system
                    .reset_ready_after_countdown(old_status, game.status);

                self.last_tick = now;
            }
        }
    }

    // Navigation methods
    fn quit(&mut self) {
        self.running = false;
    }

    fn navigate_to_start(&mut self) {
        self.screen = AppScreen::Start;
    }

    fn navigate_to_local(&mut self) {
        self.screen = AppScreen::Local;
    }

    fn navigate_to_game(&mut self) {
        self.screen = AppScreen::Game;
        self.start_local_game();
    }

    // Menu navigation
    fn menu_up(&mut self) {
        match self.screen {
            AppScreen::Start => {
                self.menu_state.start_selected = (self.menu_state.start_selected + 2 - 1) % 2;
                // Local, Quit
            }
            AppScreen::Local => {
                self.menu_state.local_selected = (self.menu_state.local_selected + 2 - 1) % 2;
                // Vs Local2, Back
            }
            _ => {}
        }
    }

    fn menu_down(&mut self) {
        match self.screen {
            AppScreen::Start => {
                self.menu_state.start_selected = (self.menu_state.start_selected + 1) % 2;
                // Local, Quit
            }
            AppScreen::Local => {
                self.menu_state.local_selected = (self.menu_state.local_selected + 1) % 2;
                // Vs Local2, Back
            }
            _ => {}
        }
    }

    fn menu_select(&mut self) {
        match self.screen {
            AppScreen::Start => {
                match self.menu_state.start_selected {
                    0 => self.events.send(AppEvent::NavigateToLocal), // Local
                    1 => self.events.send(AppEvent::Quit),            // Quit
                    _ => {}
                }
            }
            AppScreen::Local => {
                match self.menu_state.local_selected {
                    0 => self.events.send(AppEvent::NavigateToGame), // Vs Local2
                    1 => self.events.send(AppEvent::NavigateToStart), // Back
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // Terminal size management
    fn handle_resize(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);

        // Check if terminal is large enough for game
        self.ui_paused = width < MIN_GAME_WIDTH || height < MIN_GAME_HEIGHT;
    }

    pub fn calculate_centered_game_area(
        &self,
        area: ratatui::layout::Rect,
    ) -> Option<ratatui::layout::Rect> {
        if self.ui_paused {
            return None; // Too small
        }

        // Calculate centered position for fixed-size game field
        let game_width = (FIXED_GAME_WIDTH as u16).min(area.width);
        let game_height = (FIXED_GAME_HEIGHT as u16).min(area.height);

        let x_offset = area.width.saturating_sub(game_width) / 2;
        let y_offset = area.height.saturating_sub(game_height) / 2;

        Some(ratatui::layout::Rect {
            x: area.x + x_offset,
            y: area.y + y_offset,
            width: game_width,
            height: game_height,
        })
    }

    // Game methods
    fn start_local_game(&mut self) {
        let config = Config::default();
        self.game = Some(Game::new(config));
        self.input_system.reset();
        self.last_tick = Instant::now();
    }

    // Helper methods for UI
    pub fn get_start_menu_items(&self) -> Vec<&str> {
        vec!["Local", "Quit"]
    }

    pub fn get_local_menu_items(&self) -> Vec<&str> {
        vec!["Vs Local2", "Back"]
    }
}
