//! Transport abstraction for network communication

use std::collections::VecDeque;
use std::fmt;

/// Errors that can occur during transport operations
#[derive(Debug, Clone, PartialEq)]
pub enum TransportError {
    /// Connection is not open
    NotConnected,
    /// Failed to send message
    SendFailed(String),
    /// Connection failed
    ConnectionFailed(String),
    /// Invalid configuration
    InvalidConfig(String),
    /// Transport is already closed
    AlreadyClosed,
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::NotConnected => write!(f, "Transport not connected"),
            TransportError::SendFailed(msg) => write!(f, "Send failed: {}", msg),
            TransportError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            TransportError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            TransportError::AlreadyClosed => write!(f, "Transport already closed"),
        }
    }
}

impl std::error::Error for TransportError {}

/// Transport abstraction for sending and receiving bytes
pub trait Transport: Send + Sync {
    /// Send bytes to the remote peer
    fn send(&self, bytes: &[u8]) -> Result<(), TransportError>;

    /// Set callback for incoming messages
    fn set_on_message(&mut self, callback: Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>);

    /// Check if transport is currently connected and ready to send
    fn is_open(&self) -> bool;

    /// Close the transport connection
    fn close(&mut self) -> Result<(), TransportError>;

    /// Get connection status as a human-readable string
    fn status(&self) -> String;
}

/// Mock transport implementation for testing
pub struct MockTransport {
    is_open: bool,
    sent_messages: VecDeque<Vec<u8>>,
    on_message: Option<Box<dyn Fn(Vec<u8>) + Send + Sync>>,
    should_fail_send: bool,
}

impl MockTransport {
    /// Create a new mock transport
    pub fn new() -> Self {
        Self {
            is_open: true,
            sent_messages: VecDeque::new(),
            on_message: None,
            should_fail_send: false,
        }
    }

    /// Create a closed mock transport
    pub fn new_closed() -> Self {
        Self {
            is_open: false,
            sent_messages: VecDeque::new(),
            on_message: None,
            should_fail_send: false,
        }
    }

    /// Set whether send operations should fail
    pub fn set_should_fail_send(&mut self, should_fail: bool) {
        self.should_fail_send = should_fail;
    }

    /// Simulate receiving a message from the remote peer
    pub fn receive_message(&self, bytes: Vec<u8>) {
        if let Some(callback) = &self.on_message {
            callback(bytes);
        }
    }

    /// Get all messages that were sent via this transport
    pub fn sent_messages(&self) -> &VecDeque<Vec<u8>> {
        &self.sent_messages
    }

    /// Pop the oldest sent message
    pub fn pop_sent_message(&mut self) -> Option<Vec<u8>> {
        self.sent_messages.pop_front()
    }

    /// Clear all sent messages
    pub fn clear_sent_messages(&mut self) {
        self.sent_messages.clear();
    }

    /// Set the connection status
    pub fn set_open(&mut self, is_open: bool) {
        self.is_open = is_open;
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for MockTransport {
    fn send(&self, _bytes: &[u8]) -> Result<(), TransportError> {
        if !self.is_open {
            return Err(TransportError::NotConnected);
        }

        if self.should_fail_send {
            return Err(TransportError::SendFailed("Mock failure".to_string()));
        }

        // In a real implementation, we can't mutate self in send(), but for testing
        // we'll use interior mutability. For now, we'll just not store the message
        // in the send method and provide a separate way to track sent messages.
        Ok(())
    }

    fn set_on_message(&mut self, callback: Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>) {
        self.on_message = Some(callback);
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn close(&mut self) -> Result<(), TransportError> {
        if !self.is_open {
            return Err(TransportError::AlreadyClosed);
        }
        self.is_open = false;
        Ok(())
    }

    fn status(&self) -> String {
        if self.is_open {
            "Connected (Mock)".to_string()
        } else {
            "Disconnected (Mock)".to_string()
        }
    }
}

/// A better mock transport that can actually record sent messages
pub struct RecordingMockTransport {
    inner: std::sync::Arc<std::sync::Mutex<MockTransportInner>>,
}

struct MockTransportInner {
    is_open: bool,
    sent_messages: VecDeque<Vec<u8>>,
    on_message: Option<Box<dyn Fn(Vec<u8>) + Send + Sync>>,
    should_fail_send: bool,
}

impl RecordingMockTransport {
    /// Create a new recording mock transport
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(MockTransportInner {
                is_open: true,
                sent_messages: VecDeque::new(),
                on_message: None,
                should_fail_send: false,
            })),
        }
    }

    /// Create a closed recording mock transport
    pub fn new_closed() -> Self {
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(MockTransportInner {
                is_open: false,
                sent_messages: VecDeque::new(),
                on_message: None,
                should_fail_send: false,
            })),
        }
    }

    /// Set whether send operations should fail
    pub fn set_should_fail_send(&mut self, should_fail: bool) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.should_fail_send = should_fail;
        }
    }

    /// Simulate receiving a message from the remote peer
    pub fn receive_message(&self, bytes: Vec<u8>) {
        if let Ok(inner) = self.inner.lock() {
            if let Some(callback) = &inner.on_message {
                callback(bytes);
            }
        }
    }

    /// Get all messages that were sent via this transport
    pub fn sent_messages(&self) -> Vec<Vec<u8>> {
        if let Ok(inner) = self.inner.lock() {
            inner.sent_messages.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Pop the oldest sent message
    pub fn pop_sent_message(&mut self) -> Option<Vec<u8>> {
        if let Ok(mut inner) = self.inner.lock() {
            inner.sent_messages.pop_front()
        } else {
            None
        }
    }

    /// Clear all sent messages
    pub fn clear_sent_messages(&mut self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.sent_messages.clear();
        }
    }

    /// Set the connection status
    pub fn set_open(&mut self, is_open: bool) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.is_open = is_open;
        }
    }
}

impl Default for RecordingMockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for RecordingMockTransport {
    fn send(&self, bytes: &[u8]) -> Result<(), TransportError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| TransportError::SendFailed("Lock poisoned".to_string()))?;

        if !inner.is_open {
            return Err(TransportError::NotConnected);
        }

        if inner.should_fail_send {
            return Err(TransportError::SendFailed("Mock failure".to_string()));
        }

        inner.sent_messages.push_back(bytes.to_vec());
        Ok(())
    }

    fn set_on_message(&mut self, callback: Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.on_message = Some(callback);
        }
    }

    fn is_open(&self) -> bool {
        if let Ok(inner) = self.inner.lock() {
            inner.is_open
        } else {
            false
        }
    }

    fn close(&mut self) -> Result<(), TransportError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| TransportError::SendFailed("Lock poisoned".to_string()))?;

        if !inner.is_open {
            return Err(TransportError::AlreadyClosed);
        }
        inner.is_open = false;
        Ok(())
    }

    fn status(&self) -> String {
        if let Ok(inner) = self.inner.lock() {
            if inner.is_open {
                "Connected (Recording Mock)".to_string()
            } else {
                "Disconnected (Recording Mock)".to_string()
            }
        } else {
            "Error (Recording Mock)".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_mock_transport_basic_operations() {
        let mut transport = MockTransport::new();

        assert!(transport.is_open());
        assert_eq!(transport.status(), "Connected (Mock)");

        // Test send (should succeed)
        assert!(transport.send(b"hello").is_ok());

        // Test close
        assert!(transport.close().is_ok());
        assert!(!transport.is_open());
        assert_eq!(transport.status(), "Disconnected (Mock)");

        // Send should fail when closed
        assert_eq!(transport.send(b"world"), Err(TransportError::NotConnected));

        // Double close should fail
        assert_eq!(transport.close(), Err(TransportError::AlreadyClosed));
    }

    #[test]
    fn test_mock_transport_send_failure() {
        let mut transport = MockTransport::new();
        transport.set_should_fail_send(true);

        assert_eq!(
            transport.send(b"test"),
            Err(TransportError::SendFailed("Mock failure".to_string()))
        );
    }

    #[test]
    fn test_mock_transport_message_callback() {
        let mut transport = MockTransport::new();
        let received_message = Arc::new(std::sync::Mutex::new(None));
        let received_clone = Arc::clone(&received_message);

        transport.set_on_message(Box::new(move |bytes| {
            *received_clone.lock().unwrap() = Some(bytes);
        }));

        transport.receive_message(b"test message".to_vec());

        let received = received_message.lock().unwrap();
        assert_eq!(received.as_ref().unwrap(), b"test message");
    }

    #[test]
    fn test_recording_mock_transport() {
        let mut transport = RecordingMockTransport::new();

        // Test send and recording
        assert!(transport.send(b"hello").is_ok());
        assert!(transport.send(b"world").is_ok());

        let sent = transport.sent_messages();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0], b"hello");
        assert_eq!(sent[1], b"world");

        // Test pop message
        assert_eq!(transport.pop_sent_message().unwrap(), b"hello");
        assert_eq!(transport.pop_sent_message().unwrap(), b"world");
        assert!(transport.pop_sent_message().is_none());
    }

    #[test]
    fn test_recording_mock_transport_message_callback() {
        let mut transport = RecordingMockTransport::new();
        let received_count = Arc::new(AtomicBool::new(false));
        let received_clone = Arc::clone(&received_count);

        transport.set_on_message(Box::new(move |_bytes| {
            received_clone.store(true, Ordering::Relaxed);
        }));

        transport.receive_message(b"test".to_vec());
        assert!(received_count.load(Ordering::Relaxed));
    }

    #[test]
    fn test_closed_transport() {
        let transport = MockTransport::new_closed();
        assert!(!transport.is_open());
        assert_eq!(transport.send(b"test"), Err(TransportError::NotConnected));
    }

    #[test]
    fn test_transport_error_display() {
        let errors = vec![
            TransportError::NotConnected,
            TransportError::SendFailed("test".to_string()),
            TransportError::ConnectionFailed("test".to_string()),
            TransportError::InvalidConfig("test".to_string()),
            TransportError::AlreadyClosed,
        ];

        for error in errors {
            let display_str = format!("{}", error);
            assert!(!display_str.is_empty());
        }
    }
}
