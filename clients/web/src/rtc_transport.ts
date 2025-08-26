/**
 * WebRTC transport implementation for web client
 */

export interface Transport {
  send(bytes: Uint8Array): void;
  onMessage(callback: (bytes: Uint8Array) => void): void;
  isOpen(): boolean;
  close(): void;
  status(): string;
}

export enum SdpMode {
  Offer = "offer",
  Answer = "answer",
}

export class RtcTransport implements Transport {
  private peerConnection: RTCPeerConnection;
  private dataChannel?: RTCDataChannel;
  private messageCallback?: (bytes: Uint8Array) => void;
  private isConnected = false;
  private localSdp = "";
  private mode: SdpMode;

  constructor(mode: SdpMode) {
    this.mode = mode;

    // Create peer connection with no STUN/TURN servers (LAN only)
    this.peerConnection = new RTCPeerConnection({
      iceServers: [], // No external servers for LAN-only operation
    });

    this.setupPeerConnection();
  }

  private setupPeerConnection(): void {
    // Handle data channel creation/reception
    if (this.mode === SdpMode.Offer) {
      // Host creates the data channel
      this.dataChannel = this.peerConnection.createDataChannel("game", {
        ordered: true,
        maxRetransmits: 3,
      });
      this.setupDataChannelHandlers(this.dataChannel);
    } else {
      // Join will receive the data channel
      this.peerConnection.ondatachannel = (event) => {
        this.dataChannel = event.channel;
        this.setupDataChannelHandlers(this.dataChannel);
      };
    }

    // Handle connection state changes
    this.peerConnection.onconnectionstatechange = () => {
      console.log("Connection state:", this.peerConnection.connectionState);
    };

    this.peerConnection.oniceconnectionstatechange = () => {
      console.log(
        "ICE connection state:",
        this.peerConnection.iceConnectionState
      );
    };
  }

  private setupDataChannelHandlers(channel: RTCDataChannel): void {
    channel.onopen = () => {
      console.log("Data channel opened");
      this.isConnected = true;
    };

    channel.onclose = () => {
      console.log("Data channel closed");
      this.isConnected = false;
    };

    channel.onmessage = (event) => {
      if (this.messageCallback) {
        // Convert incoming data to Uint8Array
        let bytes: Uint8Array;
        if (event.data instanceof ArrayBuffer) {
          bytes = new Uint8Array(event.data);
        } else if (event.data instanceof Uint8Array) {
          bytes = event.data;
        } else {
          // Convert string or other types to bytes
          const encoder = new TextEncoder();
          bytes = encoder.encode(event.data.toString());
        }
        this.messageCallback(bytes);
      }
    };

    channel.onerror = (error) => {
      console.error("Data channel error:", error);
    };
  }

  /**
   * Create local SDP (offer or answer)
   */
  async createLocalSdp(remoteOfferSdp?: string): Promise<string> {
    try {
      if (this.mode === SdpMode.Offer) {
        // Create offer
        const offer = await this.peerConnection.createOffer();
        await this.peerConnection.setLocalDescription(offer);

        // Wait for ICE gathering to complete
        await this.waitForIceGatheringComplete();

        this.localSdp = this.peerConnection.localDescription?.sdp || "";
        return this.localSdp;
      } else {
        // Create answer (requires remote offer first)
        if (!remoteOfferSdp) {
          throw new Error("Remote offer SDP required for answer mode");
        }

        await this.peerConnection.setRemoteDescription({
          type: "offer",
          sdp: remoteOfferSdp,
        });

        const answer = await this.peerConnection.createAnswer();
        await this.peerConnection.setLocalDescription(answer);

        // Wait for ICE gathering to complete
        await this.waitForIceGatheringComplete();

        this.localSdp = this.peerConnection.localDescription?.sdp || "";
        return this.localSdp;
      }
    } catch (error) {
      throw new Error(`Failed to create ${this.mode}: ${error}`);
    }
  }

  /**
   * Wait for ICE gathering to complete with timeout
   */
  private waitForIceGatheringComplete(): Promise<void> {
    return new Promise((resolve, reject) => {
      const timeout = 10000; // 10 second timeout
      const timeoutId = setTimeout(() => {
        reject(new Error("ICE gathering timeout - connection may still work"));
      }, timeout);

      const checkGatheringState = () => {
        console.log(
          "ICE gathering state:",
          this.peerConnection.iceGatheringState
        );

        if (this.peerConnection.iceGatheringState === "complete") {
          clearTimeout(timeoutId);
          resolve();
          return;
        }

        // If gathering is still in progress, check again soon
        if (this.peerConnection.iceGatheringState === "gathering") {
          setTimeout(checkGatheringState, 100);
        }
      };

      // Handle ICE gathering state changes
      this.peerConnection.onicegatheringstatechange = () => {
        checkGatheringState();
      };

      // Start checking immediately
      checkGatheringState();
    });
  }

  /**
   * Set remote SDP (answer for host, offer for join)
   */
  async setRemoteSdp(sdp: string): Promise<void> {
    try {
      const sessionDesc: RTCSessionDescriptionInit = {
        type: this.mode === SdpMode.Offer ? "answer" : "offer",
        sdp: sdp,
      };

      await this.peerConnection.setRemoteDescription(sessionDesc);

      // If we're in answer mode and we have a remote offer, create our answer
      if (this.mode === SdpMode.Answer && sessionDesc.type === "offer") {
        const answer = await this.peerConnection.createAnswer();
        await this.peerConnection.setLocalDescription(answer);
        this.localSdp = answer.sdp || "";
      }
    } catch (error) {
      throw new Error(`Failed to set remote description: ${error}`);
    }
  }

  /**
   * Get the local SDP
   */
  getLocalSdp(): string {
    return this.localSdp;
  }

  /**
   * Get connection mode
   */
  getMode(): SdpMode {
    return this.mode;
  }

  // Transport interface implementation
  send(bytes: Uint8Array): void {
    if (!this.isConnected || !this.dataChannel) {
      throw new Error("Transport not connected");
    }

    if (this.dataChannel.readyState !== "open") {
      throw new Error("Data channel not open");
    }

    try {
      // Convert Uint8Array to ArrayBuffer for WebRTC API compatibility
      const buffer = new ArrayBuffer(bytes.length);
      const view = new Uint8Array(buffer);
      view.set(bytes);
      this.dataChannel.send(buffer);
    } catch (error) {
      throw new Error(`Failed to send message: ${error}`);
    }
  }

  onMessage(callback: (bytes: Uint8Array) => void): void {
    this.messageCallback = callback;
  }

  isOpen(): boolean {
    return this.isConnected && this.dataChannel?.readyState === "open";
  }

  close(): void {
    if (this.dataChannel) {
      this.dataChannel.close();
    }
    this.peerConnection.close();
    this.isConnected = false;
  }

  status(): string {
    if (!this.dataChannel) {
      return "Waiting for connection";
    }

    switch (this.dataChannel.readyState) {
      case "connecting":
        return "Connecting...";
      case "open":
        return "Connected";
      case "closing":
        return "Closing...";
      case "closed":
        return "Closed";
      default:
        return "Unknown";
    }
  }
}

/**
 * Factory for creating RTC transports
 */
export class RtcTransportFactory {
  /**
   * Create a new WebRTC transport with manual SDP exchange
   */
  static async createManualSdp(
    mode: SdpMode
  ): Promise<{ transport: RtcTransport; localSdp: string }> {
    const transport = new RtcTransport(mode);

    if (mode === SdpMode.Offer) {
      const localSdp = await transport.createLocalSdp();
      return { transport, localSdp };
    } else {
      // For answer mode, local SDP will be created after setting remote offer
      return { transport, localSdp: "" };
    }
  }
}
