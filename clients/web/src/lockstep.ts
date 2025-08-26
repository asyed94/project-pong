/**
 * Lockstep networking protocol for web client
 */

import { Transport } from "./rtc_transport";
import { WasmGame, Input, InputPair, GameView } from "./types";

// Wire protocol message types
const WIRE_MSG_INPUT_PAIR = 0x01;
const WIRE_MSG_SNAPSHOT = 0x02;
const WIRE_MSG_PING = 0x03;

export enum Side {
  Left = 0,
  Right = 1,
}

export interface LockstepEvent {
  type:
    | "game_advanced"
    | "peer_disconnected"
    | "pong_received"
    | "snapshot_received";
  tick?: number;
  events?: any[];
  roundTripMs?: number;
}

export interface CoreAdapter {
  step(
    tick: number,
    aAxis: number,
    aBtn: number,
    bAxis: number,
    bBtn: number
  ): any | undefined;
  viewJson(): string;
  snapshotBytes(): Uint8Array;
  restoreBytes(bytes: Uint8Array): void;
  getCurrentTick(): number;
}

/**
 * WASM game adapter for lockstep protocol
 */
export class WasmGameAdapter implements CoreAdapter {
  private game: WasmGame;

  constructor(game: WasmGame) {
    this.game = game;
  }

  step(
    tick: number,
    aAxis: number,
    aBtn: number,
    bAxis: number,
    bBtn: number
  ): any | undefined {
    const eventJson = this.game.step(tick, aAxis, aBtn, bAxis, bBtn);
    return eventJson ? JSON.parse(eventJson) : undefined;
  }

  viewJson(): string {
    return this.game.view_json();
  }

  snapshotBytes(): Uint8Array {
    return this.game.snapshot_bytes();
  }

  restoreBytes(bytes: Uint8Array): void {
    this.game.restore_bytes(bytes);
  }

  getCurrentTick(): number {
    return this.game.get_tick();
  }
}

/**
 * Wire protocol message encoding/decoding
 */
export class WireMsg {
  static encodeInputPair(inputPair: InputPair): Uint8Array {
    const buffer = new ArrayBuffer(9);
    const view = new DataView(buffer);

    view.setUint8(0, WIRE_MSG_INPUT_PAIR);
    view.setUint32(1, inputPair.tick, true); // little endian
    view.setInt8(5, inputPair.a.axis_y);
    view.setUint8(6, inputPair.a.buttons);
    view.setInt8(7, inputPair.b.axis_y);
    view.setUint8(8, inputPair.b.buttons);

    return new Uint8Array(buffer);
  }

  static encodeSnapshot(snapshotBytes: Uint8Array): Uint8Array {
    const buffer = new ArrayBuffer(1 + snapshotBytes.length);
    const view = new DataView(buffer);

    view.setUint8(0, WIRE_MSG_SNAPSHOT);
    new Uint8Array(buffer, 1).set(snapshotBytes);

    return new Uint8Array(buffer);
  }

  static encodePing(timestamp: number): Uint8Array {
    const buffer = new ArrayBuffer(5);
    const view = new DataView(buffer);

    view.setUint8(0, WIRE_MSG_PING);
    view.setUint32(1, timestamp, true); // little endian

    return new Uint8Array(buffer);
  }

  static decode(bytes: Uint8Array): any {
    if (bytes.length === 0) {
      throw new Error("Empty message");
    }

    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    const msgType = view.getUint8(0);

    switch (msgType) {
      case WIRE_MSG_INPUT_PAIR:
        if (bytes.length !== 9) {
          throw new Error("Invalid InputPair message length");
        }
        return {
          type: "input_pair",
          inputPair: {
            tick: view.getUint32(1, true),
            a: {
              axis_y: view.getInt8(5),
              buttons: view.getUint8(6),
            },
            b: {
              axis_y: view.getInt8(7),
              buttons: view.getUint8(8),
            },
          },
        };

      case WIRE_MSG_SNAPSHOT:
        return {
          type: "snapshot",
          data: new Uint8Array(
            bytes.buffer,
            bytes.byteOffset + 1,
            bytes.length - 1
          ),
        };

      case WIRE_MSG_PING:
        if (bytes.length !== 5) {
          throw new Error("Invalid Ping message length");
        }
        return {
          type: "ping",
          timestamp: view.getUint32(1, true),
        };

      default:
        throw new Error(`Unknown message type: ${msgType}`);
    }
  }
}

/**
 * Lockstep protocol implementation for web client
 */
export class Lockstep {
  private core: CoreAdapter;
  private transport: Transport;
  private currentTick: number;
  private localSide: Side;
  private isRunning = false;

  private localInputBuffer = new Map<number, Input>();
  private remoteInputBuffer = new Map<number, Input>();

  constructor(
    gameAdapter: CoreAdapter,
    transport: Transport,
    _tickHz: number,
    localSide: Side,
    _isTimekeeper: boolean
  ) {
    this.core = gameAdapter;
    this.transport = transport;
    this.localSide = localSide;
    this.currentTick = this.core.getCurrentTick();

    // Set up message handler
    this.transport.onMessage((bytes) => {
      this.onNetMessage(bytes);
    });
  }

  /**
   * Start the lockstep protocol
   */
  start(): void {
    if (!this.transport.isOpen()) {
      throw new Error("Transport not connected");
    }

    this.isRunning = true;
    this.currentTick = this.core.getCurrentTick();

    // Clear any stale buffered inputs
    this.localInputBuffer.clear();
    this.remoteInputBuffer.clear();
  }

  /**
   * Stop the lockstep protocol
   */
  stop(): void {
    this.isRunning = false;
    this.localInputBuffer.clear();
    this.remoteInputBuffer.clear();
  }

  /**
   * Submit local input for the current tick
   */
  onLocalInput(axisY: number, buttons: number): void {
    if (!this.isRunning) {
      throw new Error("Lockstep not running");
    }

    const input: Input = { axis_y: axisY, buttons };
    this.localInputBuffer.set(this.currentTick, input);

    // Send input to remote peer
    const remoteInput: Input = { axis_y: 0, buttons: 0 }; // Placeholder - we don't know remote input yet
    const inputPair: InputPair = {
      tick: this.currentTick,
      a: this.localSide === Side.Left ? input : remoteInput,
      b: this.localSide === Side.Left ? remoteInput : input,
    };

    const wireMsg = WireMsg.encodeInputPair(inputPair);
    try {
      this.transport.send(wireMsg);
    } catch (error) {
      console.error("Failed to send input:", error);
    }
  }

  /**
   * Process incoming network message
   */
  private onNetMessage(bytes: Uint8Array): LockstepEvent[] {
    if (!this.isRunning) {
      return [];
    }

    try {
      const wireMsg = WireMsg.decode(bytes);
      const events: LockstepEvent[] = [];

      switch (wireMsg.type) {
        case "input_pair": {
          // Extract the remote input for our current tick
          const remoteInput =
            this.localSide === Side.Left
              ? wireMsg.inputPair.b // We're left, so remote is right (b)
              : wireMsg.inputPair.a; // We're right, so remote is left (a)

          this.remoteInputBuffer.set(wireMsg.inputPair.tick, remoteInput);
          break;
        }

        case "snapshot": {
          this.core.restoreBytes(wireMsg.data);
          this.currentTick = this.core.getCurrentTick();

          events.push({
            type: "snapshot_received",
            tick: this.currentTick,
          });
          break;
        }

        case "ping": {
          // Respond with a pong
          const pong = WireMsg.encodePing(wireMsg.timestamp);
          try {
            this.transport.send(pong);
          } catch (error) {
            console.error("Failed to send pong:", error);
          }
          break;
        }
      }

      return events;
    } catch (error) {
      console.error("Failed to process network message:", error);
      return [];
    }
  }

  /**
   * Try to advance the simulation (call this regularly in your game loop)
   */
  tick(): { view: GameView; events: LockstepEvent[] } {
    const events: LockstepEvent[] = [];

    if (!this.isRunning) {
      // Return current view even if not running
      const viewJson = this.core.viewJson();
      return { view: JSON.parse(viewJson), events };
    }

    // Check if we have both local and remote inputs for the current tick
    const localInput = this.localInputBuffer.get(this.currentTick);
    const remoteInput = this.remoteInputBuffer.get(this.currentTick);

    if (localInput && remoteInput) {
      // Create input pair based on our side
      const aInput = this.localSide === Side.Left ? localInput : remoteInput;
      const bInput = this.localSide === Side.Left ? remoteInput : localInput;

      // Step the simulation
      const gameEvent = this.core.step(
        this.currentTick,
        aInput.axis_y,
        aInput.buttons,
        bInput.axis_y,
        bInput.buttons
      );

      // Clean up processed inputs
      this.localInputBuffer.delete(this.currentTick);
      this.remoteInputBuffer.delete(this.currentTick);

      // Advance tick
      this.currentTick += 1;

      if (gameEvent) {
        events.push({
          type: "game_advanced",
          tick: this.currentTick - 1,
          events: [gameEvent],
        });
      }
    }

    // Get current view
    const viewJson = this.core.viewJson();
    const view: GameView = JSON.parse(viewJson);

    return { view, events };
  }

  /**
   * Request a snapshot from the remote peer
   */
  requestSnapshot(): void {
    if (!this.isRunning) {
      throw new Error("Lockstep not running");
    }

    // Send our current snapshot to the peer
    const snapshotBytes = this.core.snapshotBytes();
    const wireMsg = WireMsg.encodeSnapshot(snapshotBytes);
    try {
      this.transport.send(wireMsg);
    } catch (error) {
      console.error("Failed to send snapshot:", error);
    }
  }

  /**
   * Send a ping to measure round-trip time
   */
  ping(): void {
    if (!this.isRunning) {
      throw new Error("Lockstep not running");
    }

    const timestamp = Date.now();
    const ping = WireMsg.encodePing(timestamp);
    try {
      this.transport.send(ping);
    } catch (error) {
      console.error("Failed to send ping:", error);
    }
  }

  /**
   * Get the current game view
   */
  view(): GameView {
    const viewJson = this.core.viewJson();
    return JSON.parse(viewJson);
  }

  /**
   * Get the current tick
   */
  getCurrentTick(): number {
    return this.currentTick;
  }

  /**
   * Check if we're waiting for remote input
   */
  isWaitingForRemote(): boolean {
    if (!this.isRunning) {
      return false;
    }

    return (
      this.localInputBuffer.has(this.currentTick) &&
      !this.remoteInputBuffer.has(this.currentTick)
    );
  }

  /**
   * Get transport status
   */
  transportStatus(): string {
    return this.transport.status();
  }

  /**
   * Check if transport is connected
   */
  isConnected(): boolean {
    return this.transport.isOpen();
  }

  /**
   * Get buffered input counts for debugging
   */
  getBufferInfo(): { localCount: number; remoteCount: number } {
    return {
      localCount: this.localInputBuffer.size,
      remoteCount: this.remoteInputBuffer.size,
    };
  }
}
