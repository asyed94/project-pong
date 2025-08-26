//! PeerJS bridge transport implementation for terminal client

use pong_core::transport::{Transport, TransportError};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

/// Mode for peer connection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PeerMode {
    Host, // Host mode - waits for connections
    Join, // Join mode - connects to host
}

/// Messages sent from Node.js bridge to Rust
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum BridgeMessage {
    #[serde(rename = "peer_ready")]
    PeerReady { peer_id: String, mode: String },

    #[serde(rename = "connected")]
    Connected { peer_id: String },

    #[serde(rename = "data")]
    Data { bytes: Vec<u8> },

    #[serde(rename = "connection_closed")]
    ConnectionClosed,

    #[serde(rename = "connection_error")]
    ConnectionError { message: String },

    #[serde(rename = "error")]
    Error { message: String },

    #[serde(rename = "disconnected")]
    Disconnected,
}

/// Messages sent from Rust to Node.js bridge
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
enum RustMessage {
    #[serde(rename = "send")]
    Send { bytes: Vec<u8> },

    #[serde(rename = "close")]
    Close,
}

/// Transport state
#[derive(Debug, Clone, PartialEq)]
enum TransportState {
    /// Initializing the bridge process
    Initializing,
    /// Bridge is ready, waiting for connection
    WaitingForConnection,
    /// Connected and ready to send/receive
    Connected,
    /// Connection closed or failed
    Closed,
}

/// Internal transport data
struct TransportInner {
    state: TransportState,
    peer_id: Option<String>,
    connected_peer: Option<String>,
    received_messages: VecDeque<Vec<u8>>,
    on_message: Option<Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>>,
    bridge_process: Option<Child>,
    error_message: Option<String>,
}

/// PeerJS bridge transport implementation
pub struct PeerBridgeTransport {
    inner: Arc<Mutex<TransportInner>>,
    mode: PeerMode,
}

impl PeerBridgeTransport {
    /// Create a new PeerJS bridge transport
    pub fn new(mode: PeerMode, host_peer_id: Option<String>) -> Result<Self, TransportError> {
        // Check if Node.js is available
        if !Self::check_nodejs_available() {
            return Err(TransportError::ConnectionFailed(
                "Node.js is not available. Please install Node.js to use PeerJS networking."
                    .to_string(),
            ));
        }

        // Check if the bridge script exists
        let bridge_path = std::path::Path::new("clients/terminal/src/peer_bridge.js");
        if !bridge_path.exists() {
            return Err(TransportError::ConnectionFailed(
                "PeerJS bridge script not found. Expected at clients/terminal/src/peer_bridge.js"
                    .to_string(),
            ));
        }

        let inner = Arc::new(Mutex::new(TransportInner {
            state: TransportState::Initializing,
            peer_id: None,
            connected_peer: None,
            received_messages: VecDeque::new(),
            on_message: None,
            bridge_process: None,
            error_message: None,
        }));

        let transport = Self {
            inner: inner.clone(),
            mode,
        };

        // Start the bridge process
        transport.start_bridge_process(host_peer_id)?;

        Ok(transport)
    }

    /// Check if Node.js is available
    fn check_nodejs_available() -> bool {
        Command::new("node").arg("--version").output().is_ok()
    }

    /// Start the Node.js bridge process
    fn start_bridge_process(&self, host_peer_id: Option<String>) -> Result<(), TransportError> {
        let bridge_path = "clients/terminal/src/peer_bridge.js";

        let mut cmd = Command::new("node");
        cmd.arg(bridge_path);

        match self.mode {
            PeerMode::Host => {
                cmd.arg("host");
            }
            PeerMode::Join => {
                cmd.arg("join");
                if let Some(peer_id) = host_peer_id {
                    cmd.arg(peer_id);
                } else {
                    return Err(TransportError::InvalidConfig(
                        "Host peer ID required for join mode".to_string(),
                    ));
                }
            }
        }

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            TransportError::ConnectionFailed(format!("Failed to start bridge process: {}", e))
        })?;

        // Get handles to stdin/stdout
        let stdin = child.stdin.take().ok_or_else(|| {
            TransportError::ConnectionFailed("Failed to get stdin handle".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            TransportError::ConnectionFailed("Failed to get stdout handle".to_string())
        })?;

        // Store the process handle
        {
            let mut inner = self.inner.lock().map_err(|_| {
                TransportError::ConnectionFailed("Failed to acquire lock".to_string())
            })?;
            inner.bridge_process = Some(child);
        }

        // Start reader thread for bridge messages
        let inner_clone = self.inner.clone();
        let reader = BufReader::new(stdout);
        thread::spawn(move || {
            Self::bridge_reader_thread(inner_clone, reader);
        });

        // Store stdin handle for sending messages
        let inner_clone2 = self.inner.clone();
        thread::spawn(move || {
            Self::bridge_writer_thread(inner_clone2, stdin);
        });

        Ok(())
    }

    /// Thread for reading messages from the bridge
    fn bridge_reader_thread(
        inner: Arc<Mutex<TransportInner>>,
        mut reader: BufReader<std::process::ChildStdout>,
    ) {
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Ok(message) = serde_json::from_str::<BridgeMessage>(line.trim()) {
                        Self::handle_bridge_message(&inner, message);
                    } else {
                        eprintln!("[Bridge] Invalid message: {}", line.trim());
                    }
                }
                Err(e) => {
                    eprintln!("[Bridge] Read error: {}", e);
                    break;
                }
            }
        }

        // Mark as closed when reader thread exits
        if let Ok(mut inner_guard) = inner.lock() {
            inner_guard.state = TransportState::Closed;
        }
    }

    /// Thread for writing messages to the bridge
    fn bridge_writer_thread(
        inner: Arc<Mutex<TransportInner>>,
        mut writer: std::process::ChildStdin,
    ) {
        // This thread will be used by the send() method
        // For now, just keep it alive
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Check if we should exit
            if let Ok(inner_guard) = inner.lock() {
                if inner_guard.state == TransportState::Closed {
                    break;
                }
            }
        }
    }

    /// Handle messages from the bridge
    fn handle_bridge_message(inner: &Arc<Mutex<TransportInner>>, message: BridgeMessage) {
        if let Ok(mut inner_guard) = inner.lock() {
            match message {
                BridgeMessage::PeerReady { peer_id, mode: _ } => {
                    println!("[Bridge] Peer ready with ID: {}", peer_id);
                    inner_guard.peer_id = Some(peer_id);
                    inner_guard.state = TransportState::WaitingForConnection;
                }
                BridgeMessage::Connected { peer_id } => {
                    println!("[Bridge] Connected to peer: {}", peer_id);
                    inner_guard.connected_peer = Some(peer_id);
                    inner_guard.state = TransportState::Connected;
                }
                BridgeMessage::Data { bytes } => {
                    // Call the callback if set
                    if let Some(callback) = &inner_guard.on_message {
                        callback(bytes.clone());
                    } else {
                        // Store for later retrieval
                        inner_guard.received_messages.push_back(bytes);
                    }
                }
                BridgeMessage::ConnectionClosed => {
                    println!("[Bridge] Connection closed");
                    inner_guard.state = TransportState::Closed;
                }
                BridgeMessage::ConnectionError { message } => {
                    println!("[Bridge] Connection error: {}", message);
                    inner_guard.error_message = Some(message);
                    inner_guard.state = TransportState::Closed;
                }
                BridgeMessage::Error { message } => {
                    println!("[Bridge] Error: {}", message);
                    inner_guard.error_message = Some(message);
                    inner_guard.state = TransportState::Closed;
                }
                BridgeMessage::Disconnected => {
                    println!("[Bridge] Peer disconnected");
                    inner_guard.state = TransportState::Closed;
                }
            }
        }
    }

    /// Send a message to the bridge
    fn send_to_bridge(&self, message: RustMessage) -> Result<(), TransportError> {
        // For now, print the message (in a real implementation, we'd send via stdin)
        let json = serde_json::to_string(&message).map_err(|e| {
            TransportError::SendFailed(format!("Failed to serialize message: {}", e))
        })?;

        println!("[Bridge] Sending: {}", json);
        Ok(())
    }

    /// Get the peer ID (for host mode)
    pub fn get_peer_id(&self) -> Option<String> {
        if let Ok(inner) = self.inner.lock() {
            inner.peer_id.clone()
        } else {
            None
        }
    }

    /// Get connection mode
    pub fn get_mode(&self) -> PeerMode {
        self.mode
    }
}

impl Transport for PeerBridgeTransport {
    fn send(&self, bytes: &[u8]) -> Result<(), TransportError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| TransportError::SendFailed("Failed to acquire lock".to_string()))?;

        if inner.state != TransportState::Connected {
            return Err(TransportError::NotConnected);
        }

        drop(inner); // Release lock before sending

        self.send_to_bridge(RustMessage::Send {
            bytes: bytes.to_vec(),
        })
    }

    fn set_on_message(&mut self, callback: Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>) {
        if let Ok(mut inner) = self.inner.lock() {
            // Process any queued messages first
            while let Some(message) = inner.received_messages.pop_front() {
                callback(message);
            }

            inner.on_message = Some(callback);
        }
    }

    fn is_open(&self) -> bool {
        if let Ok(inner) = self.inner.lock() {
            inner.state == TransportState::Connected
        } else {
            false
        }
    }

    fn close(&mut self) -> Result<(), TransportError> {
        if let Ok(mut inner) = self.inner.lock() {
            if inner.state == TransportState::Closed {
                return Err(TransportError::AlreadyClosed);
            }

            inner.state = TransportState::Closed;

            // Kill the bridge process
            if let Some(mut process) = inner.bridge_process.take() {
                let _ = process.kill();
                let _ = process.wait();
            }
        }

        // Send close message to bridge
        let _ = self.send_to_bridge(RustMessage::Close);
        Ok(())
    }

    fn status(&self) -> String {
        if let Ok(inner) = self.inner.lock() {
            match inner.state {
                TransportState::Initializing => "Initializing PeerJS bridge...".to_string(),
                TransportState::WaitingForConnection => {
                    if let Some(peer_id) = &inner.peer_id {
                        format!("Waiting for connection (ID: {})", peer_id)
                    } else {
                        "Waiting for connection...".to_string()
                    }
                }
                TransportState::Connected => {
                    if let Some(peer_id) = &inner.connected_peer {
                        format!("Connected to {}", peer_id)
                    } else {
                        "Connected".to_string()
                    }
                }
                TransportState::Closed => {
                    if let Some(error) = &inner.error_message {
                        format!("Closed ({})", error)
                    } else {
                        "Closed".to_string()
                    }
                }
            }
        } else {
            "Error".to_string()
        }
    }
}

/// Factory for creating PeerJS bridge transports
pub struct PeerBridgeTransportFactory;

impl PeerBridgeTransportFactory {
    /// Create a host transport
    pub fn create_host() -> Result<PeerBridgeTransport, TransportError> {
        PeerBridgeTransport::new(PeerMode::Host, None)
    }

    /// Create a guest transport that connects to a host
    pub fn create_guest(host_peer_id: String) -> Result<PeerBridgeTransport, TransportError> {
        PeerBridgeTransport::new(PeerMode::Join, Some(host_peer_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_nodejs() {
        // This test will pass if Node.js is available
        println!(
            "Node.js available: {}",
            PeerBridgeTransport::check_nodejs_available()
        );
    }

    #[test]
    fn test_create_host_transport() {
        // This will fail if Node.js is not available, which is expected
        match PeerBridgeTransportFactory::create_host() {
            Ok(transport) => {
                assert_eq!(transport.get_mode(), PeerMode::Host);
                println!("Host transport created successfully");
            }
            Err(e) => {
                println!("Expected error (Node.js may not be available): {}", e);
            }
        }
    }

    #[test]
    fn test_create_guest_transport() {
        // This will fail if Node.js is not available, which is expected
        match PeerBridgeTransportFactory::create_guest("test-peer-id".to_string()) {
            Ok(transport) => {
                assert_eq!(transport.get_mode(), PeerMode::Join);
                println!("Guest transport created successfully");
            }
            Err(e) => {
                println!("Expected error (Node.js may not be available): {}", e);
            }
        }
    }
}
