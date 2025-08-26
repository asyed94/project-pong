//! Lockstep networking protocol for synchronized multiplayer gameplay

use crate::serialization::SerializationError;
use crate::transport::{Transport, TransportError};
use crate::types::*;
use crate::wire_protocol::WireMsg;
use crate::Game;
use std::collections::HashMap;

/// Errors that can occur in lockstep protocol
#[derive(Debug, Clone, PartialEq)]
pub enum LockstepError {
    /// Transport error
    Transport(String),
    /// Serialization error
    Serialization(SerializationError),
    /// Invalid message received
    InvalidMessage(String),
    /// Game is not running
    NotRunning,
    /// Tick synchronization error
    SyncError(String),
}

impl From<TransportError> for LockstepError {
    fn from(error: TransportError) -> Self {
        LockstepError::Transport(error.to_string())
    }
}

impl From<SerializationError> for LockstepError {
    fn from(error: SerializationError) -> Self {
        LockstepError::Serialization(error)
    }
}

/// Events that can occur during lockstep processing
#[derive(Debug, Clone, PartialEq)]
pub enum LockstepEvent {
    /// Game advanced to a new tick with these events
    GameAdvanced { tick: Tick, events: Vec<Event> },
    /// Peer disconnected
    PeerDisconnected,
    /// Ping response received
    PongReceived { round_trip_ms: u32 },
    /// Snapshot received from peer
    SnapshotReceived { tick: Tick },
}

/// Core adapter trait for the game engine
pub trait CoreAdapter {
    /// Step the game simulation forward one tick
    fn step(&mut self, inputs: &InputPair) -> Option<Event>;

    /// Get the current game view for rendering
    fn view(&self) -> View;

    /// Create a snapshot of the current game state
    fn snapshot(&self) -> Snapshot;

    /// Restore game state from a snapshot
    fn restore(&mut self, snapshot: &Snapshot);

    /// Get the current tick number
    fn current_tick(&self) -> Tick;
}

/// Lockstep protocol implementation
pub struct Lockstep<C: CoreAdapter, T: Transport> {
    /// Game engine adapter
    core: C,
    /// Network transport
    transport: T,
    /// Current tick we're processing
    current_tick: Tick,
    /// Which side we are (Left or Right)
    local_side: Side,
    /// Whether we're the timekeeper (controls when to start new matches)
    is_timekeeper: bool,
    /// Target tick rate (ticks per second)
    tick_hz: u16,
    /// Buffer of local inputs waiting to be processed
    local_input_buffer: HashMap<Tick, Input>,
    /// Buffer of remote inputs waiting to be processed
    remote_input_buffer: HashMap<Tick, Input>,
    /// Last timestamp when we sent a ping
    last_ping_time: Option<std::time::Instant>,
    /// Running state
    is_running: bool,
}

impl<C: CoreAdapter, T: Transport> Lockstep<C, T> {
    /// Create a new lockstep protocol instance
    pub fn new(core: C, transport: T, tick_hz: u16, local_side: Side, is_timekeeper: bool) -> Self {
        let instance = Self {
            current_tick: core.current_tick(),
            core,
            transport,
            local_side,
            is_timekeeper,
            tick_hz,
            local_input_buffer: HashMap::new(),
            remote_input_buffer: HashMap::new(),
            last_ping_time: None,
            is_running: false,
        };

        // Set up the message handler for incoming network messages
        // Note: This is a bit tricky because we need to handle the callback
        // For now, we'll leave this for the client to handle externally

        instance
    }

    /// Start the lockstep protocol
    pub fn start(&mut self) -> Result<(), LockstepError> {
        if !self.transport.is_open() {
            return Err(LockstepError::Transport(
                "Transport not connected".to_string(),
            ));
        }

        self.is_running = true;
        self.current_tick = self.core.current_tick();

        // Clear any stale buffered inputs
        self.local_input_buffer.clear();
        self.remote_input_buffer.clear();

        Ok(())
    }

    /// Stop the lockstep protocol
    pub fn stop(&mut self) {
        self.is_running = false;
        self.local_input_buffer.clear();
        self.remote_input_buffer.clear();
    }

    /// Submit local input for the current tick
    pub fn on_local_input(&mut self, axis_y: i8, buttons: u8) -> Result<(), LockstepError> {
        if !self.is_running {
            return Err(LockstepError::NotRunning);
        }

        let input = Input::new(axis_y, buttons);
        self.local_input_buffer.insert(self.current_tick, input);

        // Send input to remote peer
        let remote_input = Input::zero(); // Placeholder - we don't know remote input yet
        let input_pair = match self.local_side {
            Side::Left => InputPair::new(self.current_tick, input, remote_input),
            Side::Right => InputPair::new(self.current_tick, remote_input, input),
        };

        let wire_msg = WireMsg::InputPair(input_pair);
        let bytes = wire_msg.encode();
        self.transport.send(&bytes)?;

        Ok(())
    }

    /// Process incoming network message
    pub fn on_net_message(&mut self, bytes: Vec<u8>) -> Result<Vec<LockstepEvent>, LockstepError> {
        if !self.is_running {
            return Ok(vec![]);
        }

        let wire_msg = WireMsg::decode(&bytes)?;
        let mut events = Vec::new();

        match wire_msg {
            WireMsg::InputPair(input_pair) => {
                // Extract the remote input for our current tick
                let remote_input = match self.local_side {
                    Side::Left => input_pair.b,  // We're left, so remote is right (b)
                    Side::Right => input_pair.a, // We're right, so remote is left (a)
                };

                self.remote_input_buffer
                    .insert(input_pair.tick, remote_input);
            }
            WireMsg::Snapshot(snapshot_data) => {
                let snapshot = Snapshot::decode(&snapshot_data)?;
                self.core.restore(&snapshot);
                self.current_tick = snapshot.tick;

                events.push(LockstepEvent::SnapshotReceived {
                    tick: snapshot.tick,
                });
            }
            WireMsg::Ping(timestamp) => {
                // Respond with a pong
                let pong = WireMsg::ping(timestamp);
                let pong_bytes = pong.encode();
                self.transport.send(&pong_bytes)?;
            }
        }

        Ok(events)
    }

    /// Try to advance the simulation (call this regularly in your game loop)
    pub fn tick(&mut self) -> Result<Vec<LockstepEvent>, LockstepError> {
        if !self.is_running {
            return Ok(vec![]);
        }

        let mut events = Vec::new();

        // Check if we have both local and remote inputs for the current tick
        if let (Some(local_input), Some(remote_input)) = (
            self.local_input_buffer.get(&self.current_tick),
            self.remote_input_buffer.get(&self.current_tick),
        ) {
            // Create input pair based on our side
            let input_pair = match self.local_side {
                Side::Left => InputPair::new(self.current_tick, *local_input, *remote_input),
                Side::Right => InputPair::new(self.current_tick, *remote_input, *local_input),
            };

            // Step the simulation
            let game_events = self.core.step(&input_pair);

            // Clean up processed inputs
            self.local_input_buffer.remove(&self.current_tick);
            self.remote_input_buffer.remove(&self.current_tick);

            // Advance tick
            self.current_tick += 1;

            if let Some(game_event) = game_events {
                events.push(LockstepEvent::GameAdvanced {
                    tick: self.current_tick - 1,
                    events: vec![game_event],
                });
            }
        }

        Ok(events)
    }

    /// Request a snapshot from the remote peer
    pub fn request_snapshot(&mut self) -> Result<(), LockstepError> {
        if !self.is_running {
            return Err(LockstepError::NotRunning);
        }

        // Send our current snapshot to the peer
        let snapshot = self.core.snapshot();
        let wire_msg = WireMsg::snapshot(&snapshot);
        let bytes = wire_msg.encode();
        self.transport.send(&bytes)?;

        Ok(())
    }

    /// Send a ping to measure round-trip time
    pub fn ping(&mut self) -> Result<(), LockstepError> {
        if !self.is_running {
            return Err(LockstepError::NotRunning);
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;

        let ping = WireMsg::ping(timestamp);
        let bytes = ping.encode();
        self.transport.send(&bytes)?;

        self.last_ping_time = Some(std::time::Instant::now());

        Ok(())
    }

    /// Get the current game view
    pub fn view(&self) -> View {
        self.core.view()
    }

    /// Get the current tick
    pub fn current_tick(&self) -> Tick {
        self.current_tick
    }

    /// Check if we're waiting for remote input
    pub fn is_waiting_for_remote(&self) -> bool {
        if !self.is_running {
            return false;
        }

        self.local_input_buffer.contains_key(&self.current_tick)
            && !self.remote_input_buffer.contains_key(&self.current_tick)
    }

    /// Get transport status
    pub fn transport_status(&self) -> String {
        self.transport.status()
    }

    /// Check if transport is connected
    pub fn is_connected(&self) -> bool {
        self.transport.is_open()
    }

    /// Get buffered input counts for debugging
    pub fn get_buffer_info(&self) -> (usize, usize) {
        (
            self.local_input_buffer.len(),
            self.remote_input_buffer.len(),
        )
    }
}

/// Simple adapter for the Game struct
pub struct GameAdapter {
    game: Game,
}

impl GameAdapter {
    pub fn new(game: Game) -> Self {
        Self { game }
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }
}

impl CoreAdapter for GameAdapter {
    fn step(&mut self, inputs: &InputPair) -> Option<Event> {
        self.game.step(inputs)
    }

    fn view(&self) -> View {
        self.game.view()
    }

    fn snapshot(&self) -> Snapshot {
        self.game.snapshot()
    }

    fn restore(&mut self, snapshot: &Snapshot) {
        self.game.restore(snapshot)
    }

    fn current_tick(&self) -> Tick {
        self.game.tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::RecordingMockTransport;

    #[test]
    fn test_lockstep_creation() {
        let game = Game::new(Config::default());
        let adapter = GameAdapter::new(game);
        let transport = RecordingMockTransport::new();

        let lockstep = Lockstep::new(adapter, transport, 60, Side::Left, true);

        assert_eq!(lockstep.current_tick(), 0);
        assert_eq!(lockstep.local_side, Side::Left);
        assert!(lockstep.is_timekeeper);
        assert!(!lockstep.is_running);
    }

    #[test]
    fn test_lockstep_start_stop() {
        let game = Game::new(Config::default());
        let adapter = GameAdapter::new(game);
        let transport = RecordingMockTransport::new();

        let mut lockstep = Lockstep::new(adapter, transport, 60, Side::Left, true);

        assert!(lockstep.start().is_ok());
        assert!(lockstep.is_running);

        lockstep.stop();
        assert!(!lockstep.is_running);
    }

    #[test]
    fn test_lockstep_start_failed_transport() {
        let game = Game::new(Config::default());
        let adapter = GameAdapter::new(game);
        let transport = RecordingMockTransport::new_closed();

        let mut lockstep = Lockstep::new(adapter, transport, 60, Side::Left, true);

        let result = lockstep.start();
        assert!(result.is_err());
        assert!(!lockstep.is_running);
    }

    #[test]
    fn test_local_input_submission() {
        let game = Game::new(Config::default());
        let adapter = GameAdapter::new(game);
        let mut transport = RecordingMockTransport::new();

        let mut lockstep = Lockstep::new(adapter, transport, 60, Side::Left, true);
        lockstep.start().unwrap();

        // Submit local input
        assert!(lockstep.on_local_input(50, 1).is_ok());

        // Check that input was buffered
        assert!(lockstep.local_input_buffer.contains_key(&0));
        assert!(lockstep.is_waiting_for_remote());

        // Check that message was sent via transport
        let sent_messages = lockstep.transport.sent_messages();
        assert_eq!(sent_messages.len(), 1);

        // Verify the sent message is an InputPair
        let wire_msg = WireMsg::decode(&sent_messages[0]).unwrap();
        match wire_msg {
            WireMsg::InputPair(input_pair) => {
                assert_eq!(input_pair.tick, 0);
                assert_eq!(input_pair.a.axis_y, 50); // We're left side
                assert_eq!(input_pair.a.buttons, 1);
            }
            _ => panic!("Expected InputPair message"),
        }
    }

    #[test]
    fn test_input_not_submitted_when_not_running() {
        let game = Game::new(Config::default());
        let adapter = GameAdapter::new(game);
        let transport = RecordingMockTransport::new();

        let mut lockstep = Lockstep::new(adapter, transport, 60, Side::Left, true);

        let result = lockstep.on_local_input(50, 1);
        assert_eq!(result, Err(LockstepError::NotRunning));
    }

    #[test]
    fn test_remote_input_processing() {
        let game = Game::new(Config::default());
        let adapter = GameAdapter::new(game);
        let transport = RecordingMockTransport::new();

        let mut lockstep = Lockstep::new(adapter, transport, 60, Side::Left, true);
        lockstep.start().unwrap();

        // Create a remote input message (from Right side's perspective)
        let remote_input = Input::new(-25, 2);
        let input_pair = InputPair::new(0, Input::zero(), remote_input); // Left=zero, Right=remote
        let wire_msg = WireMsg::InputPair(input_pair);
        let bytes = wire_msg.encode();

        let events = lockstep.on_net_message(bytes).unwrap();
        assert!(events.is_empty()); // No events from just receiving input

        // Check that remote input was buffered
        assert!(lockstep.remote_input_buffer.contains_key(&0));
        let buffered_input = lockstep.remote_input_buffer.get(&0).unwrap();
        assert_eq!(buffered_input.axis_y, -25);
        assert_eq!(buffered_input.buttons, 2);
    }

    #[test]
    fn test_simulation_advancement() {
        let game = Game::new(Config::default());
        let adapter = GameAdapter::new(game);
        let transport = RecordingMockTransport::new();

        let mut lockstep = Lockstep::new(adapter, transport, 60, Side::Left, true);
        lockstep.start().unwrap();

        let initial_tick = lockstep.current_tick();

        // Submit local input
        lockstep.on_local_input(10, 0).unwrap();

        // Submit remote input via network message
        let remote_input = Input::new(-10, 0);
        let input_pair = InputPair::new(initial_tick, Input::zero(), remote_input);
        let wire_msg = WireMsg::InputPair(input_pair);
        lockstep.on_net_message(wire_msg.encode()).unwrap();

        // Now both inputs are available, tick should advance
        let events = lockstep.tick().unwrap();

        // Check that tick advanced
        assert_eq!(lockstep.current_tick(), initial_tick + 1);

        // Input buffers should be cleaned up
        assert!(!lockstep.local_input_buffer.contains_key(&initial_tick));
        assert!(!lockstep.remote_input_buffer.contains_key(&initial_tick));

        // Should have a GameAdvanced event (but game might not emit events every tick)
        // The important thing is that we advanced the tick
        if !events.is_empty() {
            match &events[0] {
                LockstepEvent::GameAdvanced { tick, .. } => {
                    assert_eq!(*tick, initial_tick);
                }
                _ => panic!("Expected GameAdvanced event"),
            }
        }
    }

    #[test]
    fn test_no_advancement_without_both_inputs() {
        let game = Game::new(Config::default());
        let adapter = GameAdapter::new(game);
        let transport = RecordingMockTransport::new();

        let mut lockstep = Lockstep::new(adapter, transport, 60, Side::Left, true);
        lockstep.start().unwrap();

        let initial_tick = lockstep.current_tick();

        // Submit only local input
        lockstep.on_local_input(10, 0).unwrap();

        // Try to tick - should not advance without remote input
        let events = lockstep.tick().unwrap();

        assert_eq!(lockstep.current_tick(), initial_tick); // No advancement
        assert!(events.is_empty()); // No events
        assert!(lockstep.is_waiting_for_remote());
    }

    #[test]
    fn test_buffer_info() {
        let game = Game::new(Config::default());
        let adapter = GameAdapter::new(game);
        let transport = RecordingMockTransport::new();

        let mut lockstep = Lockstep::new(adapter, transport, 60, Side::Left, true);
        lockstep.start().unwrap();

        let (local_count, remote_count) = lockstep.get_buffer_info();
        assert_eq!(local_count, 0);
        assert_eq!(remote_count, 0);

        // Add some inputs
        lockstep.on_local_input(10, 0).unwrap();

        let (local_count, remote_count) = lockstep.get_buffer_info();
        assert_eq!(local_count, 1);
        assert_eq!(remote_count, 0);
    }
}
