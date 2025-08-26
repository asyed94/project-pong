use color_eyre::eyre::WrapErr;
use ratatui::crossterm::{
    event::{
        self, Event as CrosstermEvent, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    queue,
    terminal::{disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement},
};
use std::{
    io::stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// The frequency at which tick events are emitted (60 FPS for game updates)
const TICK_FPS: f64 = 60.0;

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule.
    /// Used for game updates at 60 FPS
    Tick,
    /// Crossterm events (keyboard, mouse, etc.)
    Crossterm(CrosstermEvent),
    /// Application events
    App(AppEvent),
}

/// Application events for game control
#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Quit the application
    Quit,
    /// Navigate to a screen
    NavigateToStart,
    NavigateToLocal,
    NavigateToGame,
    /// Menu navigation
    MenuUp,
    MenuDown,
    MenuSelect,
    /// Terminal resize event
    TerminalResize(u16, u16), // width, height
}

/// Terminal event handler with enhanced keyboard support
pub struct EventHandler {
    /// Event sender channel
    sender: mpsc::Sender<Event>,
    /// Event receiver channel
    receiver: mpsc::Receiver<Event>,
    /// Whether keyboard enhancements are supported
    keyboard_enhanced: bool,
}

impl EventHandler {
    /// Constructs a new instance with enhanced keyboard support
    pub fn new() -> color_eyre::Result<Self> {
        let (sender, receiver) = mpsc::channel();

        // Check for keyboard enhancement support
        let keyboard_enhanced = matches!(supports_keyboard_enhancement(), Ok(true));

        // Enable enhanced keyboard features if supported
        if keyboard_enhanced {
            enable_raw_mode()?;
            queue!(
                stdout(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                )
            )?;
        } else {
            enable_raw_mode()?;
        }

        let actor = EventThread::new(sender.clone());
        thread::spawn(move || {
            if let Err(e) = actor.run() {
                eprintln!("Event thread error: {e}");
            }
        });

        Ok(Self {
            sender,
            receiver,
            keyboard_enhanced,
        })
    }

    /// Receives an event from the sender (blocking)
    pub fn next(&self) -> color_eyre::Result<Event> {
        Ok(self.receiver.recv()?)
    }

    /// Queue an app event
    pub fn send(&mut self, app_event: AppEvent) {
        let _ = self.sender.send(Event::App(app_event));
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        // Clean up keyboard enhancements
        if self.keyboard_enhanced {
            let _ = queue!(stdout(), PopKeyboardEnhancementFlags);
        }
        let _ = disable_raw_mode();
    }
}

/// A thread that handles reading crossterm events and emitting tick events
struct EventThread {
    sender: mpsc::Sender<Event>,
}

impl EventThread {
    fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }

    fn run(self) -> color_eyre::Result<()> {
        let tick_interval = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut last_tick = Instant::now();

        loop {
            // Emit tick events at 60 FPS
            let timeout = tick_interval.saturating_sub(last_tick.elapsed());
            if timeout == Duration::ZERO {
                last_tick = Instant::now();
                self.send(Event::Tick);
            }

            // Poll for crossterm events
            if event::poll(timeout).wrap_err("failed to poll for crossterm events")? {
                let event = event::read().wrap_err("failed to read crossterm event")?;

                match event {
                    CrosstermEvent::Resize(width, height) => {
                        // Send resize event directly as app event
                        self.send(Event::App(AppEvent::TerminalResize(width, height)));
                    }
                    _ => {
                        // Send other events normally
                        self.send(Event::Crossterm(event));
                    }
                }
            }
        }
    }

    fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }
}
