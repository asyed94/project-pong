/**
 * PeerJS transport implementation for web client
 * Provides simplified P2P networking using PeerJS
 */

import { Peer, DataConnection } from "peerjs";

export interface Transport {
  send(bytes: Uint8Array): void;
  onMessage(callback: (bytes: Uint8Array) => void): void;
  isOpen(): boolean;
  close(): void;
  status(): string;
}

export enum ConnectionMode {
  Host = "host",
  Guest = "guest",
}

export class PeerTransport implements Transport {
  private peer: Peer | null = null;
  private connection: DataConnection | null = null;
  private messageCallback?: (bytes: Uint8Array) => void;
  private isConnected = false;
  private mode: ConnectionMode;
  private peerId: string = "";
  private statusMessage: string = "Initializing...";

  constructor(mode: ConnectionMode) {
    this.mode = mode;
  }

  /**
   * Initialize the peer and get the peer ID (for hosts) or connect to a peer (for guests)
   */
  async initialize(remotePeerId?: string): Promise<string> {
    return new Promise((resolve, reject) => {
      try {
        // Create peer with auto-generated ID and better local connection support
        this.peer = new Peer({
          host: "0.peerjs.com",
          port: 443,
          path: "/",
          secure: true,
          config: {
            iceServers: [
              { urls: "stun:stun.l.google.com:19302" },
              { urls: "stun:global.stun.twilio.com:3478" },
              { urls: "stun:stun1.l.google.com:19302" },
              { urls: "stun:stun2.l.google.com:19302" },
              // Add local network support
              { urls: "stun:stun3.l.google.com:19302" },
              { urls: "stun:stun4.l.google.com:19302" },
            ],
            iceCandidatePoolSize: 10,
          },
          debug: 2, // Increase debugging level
        });

        this.peer.on("open", (id) => {
          this.peerId = id;
          console.log("Peer initialized with ID:", id);

          if (this.mode === ConnectionMode.Host) {
            this.statusMessage = "Waiting for guest to connect...";
            this.setupHostListening();
            resolve(id); // Return the peer ID for the host to share
          } else {
            // Guest mode - connect to remote peer with delay
            if (!remotePeerId) {
              reject(new Error("Remote peer ID required for guest mode"));
              return;
            }
            console.log("Guest mode: waiting before attempting connection...");
            // Add small delay to ensure host is ready
            setTimeout(() => {
              this.connectToHostWithRetry(remotePeerId, 3);
            }, 1000);
            resolve(id); // Return our own peer ID (though not needed for guests)
          }
        });

        this.peer.on("error", (error) => {
          console.error("Peer error:", error);
          this.statusMessage = `Error: ${error.message}`;
          reject(error);
        });

        this.peer.on("disconnected", () => {
          console.log("Peer disconnected, attempting to reconnect...");
          this.statusMessage = "Peer disconnected, reconnecting...";
          if (this.peer && !this.peer.destroyed) {
            this.peer.reconnect();
          }
        });
      } catch (error) {
        reject(error);
      }
    });
  }

  /**
   * Set up host to listen for incoming connections
   */
  private setupHostListening(): void {
    if (!this.peer) return;

    this.peer.on("connection", (conn) => {
      console.log("Incoming connection from:", conn.peer);
      this.connection = conn;
      this.setupConnectionHandlers(conn);
    });
  }

  /**
   * Connect to the host peer with retry logic (guest mode)
   */
  private connectToHostWithRetry(hostPeerId: string, maxRetries: number): void {
    let attempts = 0;

    const attemptConnection = () => {
      attempts++;
      console.log(
        `Connection attempt ${attempts}/${maxRetries} to host:`,
        hostPeerId
      );

      if (!this.peer) {
        console.error("Peer not initialized for connection attempt");
        return;
      }

      this.statusMessage = `Connecting to host... (attempt ${attempts}/${maxRetries})`;

      this.connection = this.peer.connect(hostPeerId, {
        reliable: true, // Use reliable data channels
      });

      if (!this.connection) {
        console.error("Failed to create connection");
        return;
      }

      // Set up connection handlers with retry logic
      this.connection.on("open", () => {
        console.log("Data connection opened successfully");
        this.isConnected = true;
        this.statusMessage = "Connected";
      });

      this.connection.on("close", () => {
        console.log("Data connection closed");
        this.isConnected = false;
        this.statusMessage = "Connection closed";
      });

      this.connection.on("error", (error) => {
        console.error(`Connection attempt ${attempts} failed:`, error);
        this.isConnected = false;

        if (attempts < maxRetries) {
          console.log(
            `Retrying in 2 seconds... (${maxRetries - attempts} attempts left)`
          );
          this.statusMessage = `Connection failed, retrying in 2s... (${
            maxRetries - attempts
          } left)`;
          setTimeout(attemptConnection, 2000);
        } else {
          console.error("All connection attempts failed");
          this.statusMessage = `Connection failed after ${maxRetries} attempts: ${error.message}`;
        }
      });

      this.setupConnectionHandlers(this.connection);

      // Set a timeout for the connection attempt
      setTimeout(() => {
        if (!this.isConnected && attempts < maxRetries) {
          console.log("Connection timeout, retrying...");
          this.connection?.close();
          setTimeout(attemptConnection, 1000);
        }
      }, 10000);
    };

    attemptConnection();
  }

  /**
   * Connect to the host peer (guest mode) - legacy method
   */
  private connectToHost(hostPeerId: string): void {
    this.connectToHostWithRetry(hostPeerId, 1);
  }

  /**
   * Set up event handlers for the data connection
   */
  private setupConnectionHandlers(conn: DataConnection): void {
    conn.on("open", () => {
      console.log("Data connection opened");
      this.isConnected = true;
      this.statusMessage = "Connected";
    });

    conn.on("close", () => {
      console.log("Data connection closed");
      this.isConnected = false;
      this.statusMessage = "Connection closed";
    });

    conn.on("error", (error) => {
      console.error("Connection error:", error);
      this.isConnected = false;
      this.statusMessage = `Connection error: ${error.message}`;
    });

    conn.on("data", (data) => {
      try {
        // Convert incoming data to Uint8Array
        let bytes: Uint8Array;

        if (data instanceof ArrayBuffer) {
          bytes = new Uint8Array(data);
        } else if (data instanceof Uint8Array) {
          bytes = data;
        } else if (Array.isArray(data)) {
          // Handle array of numbers
          bytes = new Uint8Array(data);
        } else {
          // Convert other types to string then to bytes
          const encoder = new TextEncoder();
          bytes = encoder.encode(JSON.stringify(data));
        }

        if (this.messageCallback) {
          this.messageCallback(bytes);
        }
      } catch (error) {
        console.error("Error processing received data:", error);
      }
    });
  }

  /**
   * Get the current peer ID
   */
  getPeerId(): string {
    return this.peerId;
  }

  /**
   * Get connection mode
   */
  getMode(): ConnectionMode {
    return this.mode;
  }

  // Transport interface implementation
  send(bytes: Uint8Array): void {
    if (!this.isConnected || !this.connection) {
      throw new Error("Transport not connected");
    }

    try {
      // Send as ArrayBuffer for binary data
      const buffer = bytes.buffer.slice(
        bytes.byteOffset,
        bytes.byteOffset + bytes.byteLength
      );
      this.connection.send(buffer);
    } catch (error) {
      throw new Error(`Failed to send message: ${error}`);
    }
  }

  onMessage(callback: (bytes: Uint8Array) => void): void {
    this.messageCallback = callback;
  }

  isOpen(): boolean {
    return this.isConnected && this.connection?.open === true;
  }

  close(): void {
    if (this.connection) {
      this.connection.close();
      this.connection = null;
    }

    if (this.peer) {
      this.peer.destroy();
      this.peer = null;
    }

    this.isConnected = false;
    this.statusMessage = "Closed";
  }

  status(): string {
    return this.statusMessage;
  }
}

/**
 * Factory for creating PeerJS transports
 */
export class PeerTransportFactory {
  /**
   * Create a new PeerJS transport for hosting
   */
  static async createHost(): Promise<{
    transport: PeerTransport;
    peerId: string;
  }> {
    const transport = new PeerTransport(ConnectionMode.Host);
    const peerId = await transport.initialize();
    return { transport, peerId };
  }

  /**
   * Create a new PeerJS transport for joining
   */
  static async createGuest(hostPeerId: string): Promise<PeerTransport> {
    const transport = new PeerTransport(ConnectionMode.Guest);
    await transport.initialize(hostPeerId);
    return transport;
  }
}

/**
 * Utility functions for peer ID handling
 */
export class PeerIdUtils {
  /**
   * Validate a peer ID format
   */
  static isValidPeerId(peerId: string): boolean {
    // PeerJS generates IDs that are typically 32+ characters long
    // Allow proper range for PeerJS-generated IDs
    return /^[a-zA-Z0-9_-]{8,64}$/.test(peerId);
  }

  /**
   * Format peer ID for display (add dashes for readability)
   */
  static formatPeerId(peerId: string): string {
    if (peerId.length >= 8) {
      // Split into groups of 4 characters
      return peerId.match(/.{1,4}/g)?.join("-") || peerId;
    }
    return peerId;
  }

  /**
   * Clean peer ID input (remove spaces, dashes, etc.)
   */
  static cleanPeerId(input: string): string {
    return input.replace(/[\s-]/g, "").toLowerCase();
  }

  /**
   * Generate a user-friendly connection string
   */
  static createConnectionString(peerId: string, gameMode?: string): string {
    const formatted = this.formatPeerId(peerId);
    const mode = gameMode ? ` (${gameMode})` : "";
    return `Game ID: ${formatted}${mode}`;
  }
}
