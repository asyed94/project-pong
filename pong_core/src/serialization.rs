//! Compact binary serialization for network protocol.

use crate::types::*;

/// Serialization errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationError {
    /// Buffer too small for serialization
    BufferTooSmall,
    /// Invalid data during deserialization
    InvalidData,
    /// Unexpected end of buffer
    UnexpectedEnd,
}

impl Input {
    /// Serialize to 2 bytes: [axis_y: i8, buttons: u8]
    pub fn encode(&self) -> [u8; 2] {
        [
            self.axis_y as u8, // Cast i8 to u8 preserving bit pattern
            self.buttons,
        ]
    }

    /// Deserialize from 2 bytes
    pub fn decode(bytes: &[u8]) -> Result<Self, SerializationError> {
        if bytes.len() < 2 {
            return Err(SerializationError::UnexpectedEnd);
        }

        Ok(Input {
            axis_y: bytes[0] as i8, // Cast u8 back to i8
            buttons: bytes[1],
        })
    }
}

impl InputPair {
    /// Serialize to 9 bytes: [tick: u32, a_axis: i8, a_buttons: u8, b_axis: i8, b_buttons: u8]
    pub fn encode(&self) -> [u8; 9] {
        let mut bytes = [0u8; 9];

        // Tick as little-endian u32
        bytes[0..4].copy_from_slice(&self.tick.to_le_bytes());

        // Input A
        bytes[4] = self.a.axis_y as u8;
        bytes[5] = self.a.buttons;

        // Input B
        bytes[6] = self.b.axis_y as u8;
        bytes[7] = self.b.buttons;

        // Byte 8 reserved for future use
        bytes[8] = 0;

        bytes
    }

    /// Deserialize from 9 bytes
    pub fn decode(bytes: &[u8]) -> Result<Self, SerializationError> {
        if bytes.len() < 9 {
            return Err(SerializationError::UnexpectedEnd);
        }

        let tick = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let a = Input {
            axis_y: bytes[4] as i8,
            buttons: bytes[5],
        };

        let b = Input {
            axis_y: bytes[6] as i8,
            buttons: bytes[7],
        };

        Ok(InputPair { tick, a, b })
    }
}

impl Snapshot {
    /// Serialize snapshot to compact binary format
    /// Layout: [tick:4][status:3][paddles:16][ball:16][score:2][rng:8] = 49 bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(49);

        // Tick (4 bytes)
        bytes.extend_from_slice(&self.tick.to_le_bytes());

        // Status (3 bytes: discriminant + data)
        match self.status {
            Status::Lobby => {
                bytes.push(0);
                bytes.push(0);
                bytes.push(0);
            }
            Status::Countdown(ticks) => {
                bytes.push(1);
                bytes.extend_from_slice(&ticks.to_le_bytes());
            }
            Status::Playing => {
                bytes.push(2);
                bytes.push(0);
                bytes.push(0);
            }
            Status::Scored(side, ticks) => {
                bytes.push(3);
                bytes.push(match side {
                    Side::Left => 0,
                    Side::Right => 1,
                });
                bytes.extend_from_slice(&ticks.to_le_bytes());
            }
            Status::GameOver(side) => {
                bytes.push(4);
                bytes.push(match side {
                    Side::Left => 0,
                    Side::Right => 1,
                });
                bytes.push(0);
            }
        }

        // Paddles (16 bytes: 2 * (y:4 + vy:4))
        for paddle in &self.paddles {
            bytes.extend_from_slice(&paddle.y.to_le_bytes());
            bytes.extend_from_slice(&paddle.vy.to_le_bytes());
        }

        // Ball (16 bytes: pos(8) + vel(8))
        bytes.extend_from_slice(&self.ball.pos.x.to_le_bytes());
        bytes.extend_from_slice(&self.ball.pos.y.to_le_bytes());
        bytes.extend_from_slice(&self.ball.vel.x.to_le_bytes());
        bytes.extend_from_slice(&self.ball.vel.y.to_le_bytes());

        // Score (2 bytes)
        bytes.extend_from_slice(&self.score);

        // RNG state (8 bytes)
        bytes.extend_from_slice(&self.rng.to_le_bytes());

        bytes
    }

    /// Deserialize snapshot from binary format
    pub fn decode(bytes: &[u8]) -> Result<Self, SerializationError> {
        if bytes.len() < 49 {
            return Err(SerializationError::UnexpectedEnd);
        }

        let mut offset = 0;

        // Tick
        let tick = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        // Status
        let status = match bytes[offset] {
            0 => Status::Lobby,
            1 => {
                let ticks = u16::from_le_bytes([bytes[offset + 1], bytes[offset + 2]]);
                Status::Countdown(ticks)
            }
            2 => Status::Playing,
            3 => {
                let side = match bytes[offset + 1] {
                    0 => Side::Left,
                    1 => Side::Right,
                    _ => return Err(SerializationError::InvalidData),
                };
                let ticks = u16::from_le_bytes([bytes[offset + 2], 0]); // Only lower byte used
                Status::Scored(side, ticks)
            }
            4 => {
                let side = match bytes[offset + 1] {
                    0 => Side::Left,
                    1 => Side::Right,
                    _ => return Err(SerializationError::InvalidData),
                };
                Status::GameOver(side)
            }
            _ => return Err(SerializationError::InvalidData),
        };
        offset += 3;

        // Paddles
        let mut paddles = [Paddle::new(0); 2];
        for i in 0..2 {
            let y = i32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offset += 4;
            let vy = i32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offset += 4;
            paddles[i] = Paddle { y, vy };
        }

        // Ball
        let ball_pos_x = i32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;
        let ball_pos_y = i32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;
        let ball_vel_x = i32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;
        let ball_vel_y = i32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        let ball = Ball {
            pos: Vec2::new(ball_pos_x, ball_pos_y),
            vel: Vec2::new(ball_vel_x, ball_vel_y),
        };

        // Score
        let score = [bytes[offset], bytes[offset + 1]];
        offset += 2;

        // RNG state
        let rng = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);

        Ok(Snapshot {
            tick,
            status,
            paddles,
            ball,
            score,
            rng,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_serialization() {
        let input = Input::new(-100, 5);
        let encoded = input.encode();
        let decoded = Input::decode(&encoded).unwrap();

        assert_eq!(input, decoded);
        assert_eq!(encoded.len(), 2);
    }

    #[test]
    fn test_input_serialization_edge_cases() {
        // Test extreme values
        let input_min = Input::new(-127, 0);
        let input_max = Input::new(127, 255);

        assert_eq!(input_min, Input::decode(&input_min.encode()).unwrap());
        assert_eq!(input_max, Input::decode(&input_max.encode()).unwrap());
    }

    #[test]
    fn test_input_pair_serialization() {
        let pair = InputPair::new(12345, Input::new(-50, 1), Input::new(75, 2));

        let encoded = pair.encode();
        let decoded = InputPair::decode(&encoded).unwrap();

        assert_eq!(pair, decoded);
        assert_eq!(encoded.len(), 9);
    }

    #[test]
    fn test_input_decode_insufficient_data() {
        assert_eq!(Input::decode(&[]), Err(SerializationError::UnexpectedEnd));
        assert_eq!(Input::decode(&[1]), Err(SerializationError::UnexpectedEnd));

        assert_eq!(
            InputPair::decode(&[1, 2, 3, 4]),
            Err(SerializationError::UnexpectedEnd)
        );
    }

    #[test]
    fn test_snapshot_serialization() {
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

        let encoded = snapshot.encode();
        let decoded = Snapshot::decode(&encoded).unwrap();

        assert_eq!(snapshot.tick, decoded.tick);
        assert_eq!(snapshot.status, decoded.status);
        assert_eq!(snapshot.paddles, decoded.paddles);
        assert_eq!(snapshot.ball.pos, decoded.ball.pos);
        assert_eq!(snapshot.ball.vel, decoded.ball.vel);
        assert_eq!(snapshot.score, decoded.score);
        assert_eq!(snapshot.rng, decoded.rng);
    }

    #[test]
    fn test_snapshot_all_status_variants() {
        let statuses = [
            Status::Lobby,
            Status::Countdown(180),
            Status::Playing,
            Status::Scored(Side::Left, 120),
            Status::GameOver(Side::Right),
        ];

        for status in statuses {
            let snapshot = Snapshot {
                tick: 100,
                status,
                paddles: [Paddle::new(0), Paddle::new(0)],
                ball: Ball::new(Vec2::zero(), Vec2::zero()),
                score: [0, 0],
                rng: 0,
            };

            let encoded = snapshot.encode();
            let decoded = Snapshot::decode(&encoded).unwrap();
            assert_eq!(snapshot.status, decoded.status);
        }
    }

    #[test]
    fn test_snapshot_decode_insufficient_data() {
        let short_data = vec![0u8; 10]; // Too short
        assert_eq!(
            Snapshot::decode(&short_data),
            Err(SerializationError::UnexpectedEnd)
        );
    }

    #[test]
    fn test_snapshot_decode_invalid_status() {
        let mut data = vec![0u8; 49];
        data[4] = 99; // Invalid status discriminant
        assert_eq!(
            Snapshot::decode(&data),
            Err(SerializationError::InvalidData)
        );
    }

    #[test]
    fn test_snapshot_decode_invalid_side() {
        let mut data = vec![0u8; 49];
        data[4] = 3; // Status::Scored
        data[5] = 99; // Invalid side
        assert_eq!(
            Snapshot::decode(&data),
            Err(SerializationError::InvalidData)
        );
    }
}
