// TypeScript type definitions mirroring Rust types for type safety

export interface GameView {
  tick: number;
  status: GameStatus;
  score: [number, number];
  left_paddle_y: number;
  right_paddle_y: number;
  paddle_half_h: number;
  ball_pos: { x: number; y: number };
  paddle_x_offset: number;
  paddle_width: number;
  ball_radius: number;
}

export type GameStatus =
  | "Lobby"
  | { Countdown: number }
  | "Playing"
  | { Scored: [string, number] }
  | { GameOver: string };

export interface GameEvent {
  Scored: {
    scorer: "Left" | "Right";
    score: [number, number];
  };
}

export interface WasmGame {
  new (config_json: string): WasmGame;
  step(
    tick: number,
    a_axis: number,
    a_btn: number,
    b_axis: number,
    b_btn: number
  ): string | undefined;
  view_json(): string;
  snapshot_bytes(): Uint8Array;
  restore_bytes(bytes: Uint8Array): void;
  reset_match(): void;
  get_tick(): number;
  is_active(): boolean;
  status_string(): string;
}

export interface Config {
  paddle_half_h: number;
  paddle_speed: number;
  ball_speed: number;
  ball_speed_up: number;
  wall_thickness: number;
  paddle_x: number;
  max_score: number;
  seed: number;
  tick_hz: number;
  ball_radius: number;
  paddle_width: number;
}

// Input types
export interface Input {
  axis_y: number; // [-127, 127]
  buttons: number; // Bitfield
}

export interface InputPair {
  tick: number;
  a: Input; // Left player
  b: Input; // Right player
}
