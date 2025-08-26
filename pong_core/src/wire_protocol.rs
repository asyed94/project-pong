//! Wire protocol for network communication between clients

use crate::serialization::SerializationError;
use crate::types::*;

/// Network messages that can be sent between clients
#[derive(Debug, Clone, PartialEq)]
pub enum WireMsg {
    /// Input pair for a specific tick
    InputPair(InputPair),
    /// Game state snapshot for synchronization
    Snapshot(Vec<u8>),
    /// Ping message with client timestamp
    Ping(u32),
}

impl WireMsg {
    /// Encode wire message to bytes with type header
    pub fn encode(&self) -> Vec<u8> {
        match self {
            WireMsg::InputPair(pair) => {
                let mut bytes = Vec::with_capacity(10);
                bytes.push(0x01); // Type header for InputPair
                bytes.extend_from_slice(&pair.encode());
                bytes
            }
            WireMsg::Snapshot(data) => {
                let mut bytes = Vec::with_capacity(1 + data.len());
                bytes.push(0x02); // Type header for Snapshot
                bytes.extend_from_slice(data);
                bytes
            }
            WireMsg::Ping(timestamp) => {
                let mut bytes = Vec::with_capacity(5);
                bytes.push(0x03); // Type header for Ping
                bytes.extend_from_slice(&timestamp.to_le_bytes());
                bytes
            }
        }
    }

    /// Decode wire message from bytes
    pub fn decode(bytes: &[u8]) -> Result<Self, SerializationError> {
        if bytes.is_empty() {
            return Err(SerializationError::UnexpectedEnd);
        }

        match bytes[0] {
            0x01 => {
                // InputPair message
                if bytes.len() < 10 {
                    return Err(SerializationError::UnexpectedEnd);
                }
                let pair = InputPair::decode(&bytes[1..])?;
                Ok(WireMsg::InputPair(pair))
            }
            0x02 => {
                // Snapshot message
                if bytes.len() < 2 {
                    return Err(SerializationError::UnexpectedEnd);
                }
                let snapshot_data = bytes[1..].to_vec();
                Ok(WireMsg::Snapshot(snapshot_data))
            }
            0x03 => {
                // Ping message
                if bytes.len() < 5 {
                    return Err(SerializationError::UnexpectedEnd);
                }
                let timestamp = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Ok(WireMsg::Ping(timestamp))
            }
            _ => Err(SerializationError::InvalidData),
        }
    }

    /// Get the message type as a string for debugging
    pub fn message_type(&self) -> &'static str {
        match self {
            WireMsg::InputPair(_) => "InputPair",
            WireMsg::Snapshot(_) => "Snapshot",
            WireMsg::Ping(_) => "Ping",
        }
    }

    /// Get the size of the encoded message in bytes
    pub fn encoded_size(&self) -> usize {
        match self {
            WireMsg::InputPair(_) => 10, // 1 byte header + 9 bytes InputPair
            WireMsg::Snapshot(data) => 1 + data.len(), // 1 byte header + snapshot data
            WireMsg::Ping(_) => 5,       // 1 byte header + 4 bytes timestamp
        }
    }
}

/// Utility functions for working with wire messages
impl WireMsg {
    /// Create an InputPair message
    pub fn input_pair(tick: Tick, a: Input, b: Input) -> Self {
        WireMsg::InputPair(InputPair::new(tick, a, b))
    }

    /// Create a Snapshot message from a snapshot
    pub fn snapshot(snapshot: &Snapshot) -> Self {
        WireMsg::Snapshot(snapshot.encode())
    }

    /// Create a Ping message with current timestamp (in milliseconds)
    pub fn ping(timestamp_ms: u32) -> Self {
        WireMsg::Ping(timestamp_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_pair_message() {
        let input_a = Input::new(-50, 1);
        let input_b = Input::new(75, 2);
        let pair = InputPair::new(12345, input_a, input_b);
        let msg = WireMsg::InputPair(pair);

        let encoded = msg.encode();
        let decoded = WireMsg::decode(&encoded).unwrap();

        assert_eq!(msg, decoded);
        assert_eq!(msg.message_type(), "InputPair");
        assert_eq!(msg.encoded_size(), 10);
        assert_eq!(encoded[0], 0x01); // Check type header
        assert_eq!(encoded.len(), 10);
    }

    #[test]
    fn test_snapshot_message() {
        let snapshot = Snapshot {
            tick: 1000,
            status: Status::Playing,
            paddles: [
                Paddle {
                    y: FX_ONE / 2,
                    vy: FX_ONE / 4,
                },
                Paddle {
                    y: FX_ONE / 3,
                    vy: -FX_ONE / 8,
                },
            ],
            ball: Ball {
                pos: Vec2::new(FX_ONE / 2, FX_ONE / 4),
                vel: Vec2::new(FX_ONE / 8, -FX_ONE / 16),
            },
            score: [3, 2],
            rng: 0xDEADBEEF_CAFEBABE,
        };

        let msg = WireMsg::snapshot(&snapshot);
        let encoded = msg.encode();
        let decoded = WireMsg::decode(&encoded).unwrap();

        assert_eq!(msg, decoded);
        assert_eq!(msg.message_type(), "Snapshot");
        assert_eq!(encoded[0], 0x02); // Check type header
        assert_eq!(encoded.len(), 50); // 1 byte header + 49 bytes snapshot

        // Verify we can decode the snapshot data
        if let WireMsg::Snapshot(data) = decoded {
            let decoded_snapshot = Snapshot::decode(&data).unwrap();
            assert_eq!(snapshot.tick, decoded_snapshot.tick);
            assert_eq!(snapshot.status, decoded_snapshot.status);
        } else {
            panic!("Expected Snapshot message");
        }
    }

    #[test]
    fn test_ping_message() {
        let timestamp = 0x12345678;
        let msg = WireMsg::ping(timestamp);

        let encoded = msg.encode();
        let decoded = WireMsg::decode(&encoded).unwrap();

        assert_eq!(msg, decoded);
        assert_eq!(msg.message_type(), "Ping");
        assert_eq!(msg.encoded_size(), 5);
        assert_eq!(encoded[0], 0x03); // Check type header
        assert_eq!(encoded.len(), 5);

        if let WireMsg::Ping(decoded_timestamp) = decoded {
            assert_eq!(timestamp, decoded_timestamp);
        } else {
            panic!("Expected Ping message");
        }
    }

    #[test]
    fn test_utility_constructors() {
        let input_a = Input::new(-100, 3);
        let input_b = Input::new(50, 7);
        let tick = 999;

        let msg = WireMsg::input_pair(tick, input_a, input_b);
        if let WireMsg::InputPair(pair) = msg {
            assert_eq!(pair.tick, tick);
            assert_eq!(pair.a, input_a);
            assert_eq!(pair.b, input_b);
        } else {
            panic!("Expected InputPair message");
        }
    }

    #[test]
    fn test_decode_errors() {
        // Empty buffer
        assert_eq!(WireMsg::decode(&[]), Err(SerializationError::UnexpectedEnd));

        // Invalid type header
        assert_eq!(
            WireMsg::decode(&[0xFF]),
            Err(SerializationError::InvalidData)
        );

        // InputPair too short
        assert_eq!(
            WireMsg::decode(&[0x01, 1, 2, 3]),
            Err(SerializationError::UnexpectedEnd)
        );

        // Snapshot too short
        assert_eq!(
            WireMsg::decode(&[0x02]),
            Err(SerializationError::UnexpectedEnd)
        );

        // Ping too short
        assert_eq!(
            WireMsg::decode(&[0x03, 1, 2]),
            Err(SerializationError::UnexpectedEnd)
        );
    }

    #[test]
    fn test_message_roundtrip_all_types() {
        let messages = vec![
            WireMsg::input_pair(42, Input::new(-127, 255), Input::new(127, 0)),
            WireMsg::snapshot(&Snapshot {
                tick: 0,
                status: Status::Lobby,
                paddles: [Paddle::new(0), Paddle::new(0)],
                ball: Ball::new(Vec2::zero(), Vec2::zero()),
                score: [0, 0],
                rng: 0,
            }),
            WireMsg::ping(0xFFFFFFFF),
        ];

        for msg in messages {
            let encoded = msg.encode();
            let decoded = WireMsg::decode(&encoded).unwrap();
            assert_eq!(msg, decoded);
        }
    }

    #[test]
    fn test_encoded_sizes() {
        let input_msg = WireMsg::input_pair(0, Input::zero(), Input::zero());
        assert_eq!(input_msg.encoded_size(), input_msg.encode().len());

        let snapshot = Snapshot {
            tick: 0,
            status: Status::Lobby,
            paddles: [Paddle::new(0), Paddle::new(0)],
            ball: Ball::new(Vec2::zero(), Vec2::zero()),
            score: [0, 0],
            rng: 0,
        };
        let snapshot_msg = WireMsg::snapshot(&snapshot);
        assert_eq!(snapshot_msg.encoded_size(), snapshot_msg.encode().len());

        let ping_msg = WireMsg::ping(123);
        assert_eq!(ping_msg.encoded_size(), ping_msg.encode().len());
    }
}
