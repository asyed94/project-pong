// Main entry point for Pong Web Client

import { GameStateManager } from "./GameStateManager.js";
import { GameRenderer } from "./GameRenderer.js";
import { InputManager } from "./InputManager.js";
import { MobileController } from "./MobileController.js";
import {
  initializeModeSelection,
  initializeGameOverScreen,
  initializeHostScreen,
  initializeJoinScreen,
  updateHostScreen,
  updateJoinScreen,
  GameMode,
  screenManager,
  Screen,
} from "./screens.js";
import {
  PeerTransportFactory,
  PeerTransport,
  ConnectionMode,
} from "./peer_transport.js";
import { setupHostQRDisplay, setupGuestQRScanning } from "./peer_qr_utils.js";
import { Lockstep, WasmGameAdapter, Side } from "./lockstep.js";

// Global instances
let gameStateManager: GameStateManager;
let gameRenderer: GameRenderer;
let inputManager: InputManager;
let mobileController: MobileController;

// Networking instances
let lockstep: Lockstep | null = null;
let currentTransport: any = null;
let networkingMode: "local" | "host" | "join" = "local";

async function main() {
  const appElement = document.getElementById("app");

  if (!appElement) {
    console.error("App element not found");
    return;
  }

  try {
    console.log("Starting Pong Web Client...");

    // Initialize mobile controller first
    mobileController = new MobileController();
    mobileController.initialize();

    // Initialize core modules
    gameStateManager = new GameStateManager();
    gameRenderer = new GameRenderer();
    inputManager = new InputManager();

    // Initialize WASM
    await gameStateManager.initialize();

    // Initialize input systems
    inputManager.initialize();

    // Wire up dependencies with proper input processing
    gameStateManager.setInputStateProvider((currentTime: number) => {
      const rawInput = inputManager.getInputState();
      return gameStateManager.processInputForGameMode(rawInput, currentTime);
    });

    // Set up FAB visibility callback
    inputManager.setFABVisibilityCallback(() => {
      const view = gameStateManager.getCurrentView();
      const status = view?.status;
      const statusString = typeof status === "string" ? status : "Playing";
      inputManager.updateFABVisibility(statusString);
    });

    // Initialize screen management
    initializeScreenManagement();

    // Initialize networking screens
    initializeNetworkingScreens();

    // Start game loop for rendering
    startRenderLoop();

    console.log("Pong Web Client initialized successfully");
    console.log("Ready for mode selection!");
  } catch (error) {
    console.error("Failed to initialize Pong Web Client:", error);

    // Show error in mode selection screen
    const modeTitle = document.querySelector(".mode-title") as HTMLElement;
    if (modeTitle) {
      modeTitle.textContent = `Error: ${
        error instanceof Error ? error.message : "Unknown error"
      }`;
      modeTitle.style.color = "#FF6B6B";
    }

    // Add error class to app
    appElement.classList.add("error");
  }
}

/**
 * Initialize screen management and mode selection
 */
function initializeScreenManagement(): void {
  // Initialize mode selection with networking support
  initializeModeSelection((mode) => {
    if (mode === GameMode.Host) {
      initializeHostMode();
      return;
    }

    if (mode === GameMode.Join) {
      initializeJoinMode();
      return;
    }

    // Local modes
    networkingMode = "local";
    gameStateManager.updateControlInstructions(mode);
    gameStateManager.startGame(mode);
  });

  // Initialize game over screen
  initializeGameOverScreen(
    () => {
      // Rematch
      if (networkingMode === "local") {
        gameStateManager.rematch();
      } else {
        // For networked games, implement rematch via lockstep
        console.log("Networked rematch not yet implemented");
        gameStateManager.stopGame();
      }
    },
    () => {
      // Back to menu
      cleanupNetworking();
      gameStateManager.stopGame();
    }
  );

  // Back to menu button in game screen
  const backToMenuBtn = document.getElementById("back-to-menu-btn");
  if (backToMenuBtn) {
    backToMenuBtn.addEventListener("click", () => {
      cleanupNetworking();
      gameStateManager.stopGame();
    });
  }

  // Handle ESC key for menu
  document.addEventListener("keydown", (event) => {
    if (event.code === "Escape") {
      if (gameStateManager.isGameRunning()) {
        cleanupNetworking();
        gameStateManager.stopGame();
      } else if (
        screenManager.currentScreen === Screen.Host ||
        screenManager.currentScreen === Screen.Join
      ) {
        cleanupNetworking();
        screenManager.showScreen(Screen.ModeSelect);
      }
    }
  });
}

/**
 * Initialize networking screens for Host/Join
 */
function initializeNetworkingScreens(): void {
  // Initialize host screen (just back button now - PeerJS handles connection automatically)
  initializeHostScreen(() => {
    cleanupNetworking();
  });

  // Initialize join screen with peer ID connection
  initializeJoinScreen(
    async (hostPeerId: string) => {
      try {
        updateJoinScreen("Connecting to host...");

        // Create guest transport and connect to host
        const transport = await PeerTransportFactory.createGuest(hostPeerId);
        currentTransport = transport;
        networkingMode = "join";

        // Monitor for connection
        const checkConnection = () => {
          if (transport.isOpen()) {
            updateJoinScreen("Connected! Starting game...");
            startNetworkedGame(Side.Right); // Join is right side
          } else {
            setTimeout(checkConnection, 500);
          }
        };

        setTimeout(checkConnection, 1000);
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        updateJoinScreen(`Connection failed: ${errorMessage}`);
      }
    },
    () => {
      cleanupNetworking();
    }
  );

  // Add copy button functionality for host peer ID
  const copyButton = document.getElementById("copy-peer-id-btn");
  if (copyButton) {
    copyButton.addEventListener("click", () => {
      const peerIdInput = document.getElementById(
        "host-peer-id-output"
      ) as HTMLInputElement;
      if (peerIdInput && peerIdInput.value) {
        navigator.clipboard
          .writeText(peerIdInput.value)
          .then(() => {
            // Show temporary feedback
            const originalText = copyButton.textContent;
            copyButton.textContent = "✅ Copied!";
            setTimeout(() => {
              copyButton.textContent = originalText;
            }, 2000);
          })
          .catch(() => {
            // Fallback: select the text
            peerIdInput.select();
            peerIdInput.setSelectionRange(0, 99999);
          });
      }
    });
  }
}

/**
 * Start a networked game using lockstep protocol
 */
async function startNetworkedGame(localSide: Side): Promise<void> {
  try {
    if (!currentTransport) {
      throw new Error("No transport available");
    }

    // Initialize WASM game
    const config = {
      paddle_half_h: 0.125,
      paddle_speed: 1.5,
      ball_speed: 0.5,
      ball_speed_up: 1.05,
      wall_thickness: 0.0,
      paddle_x: 0.05,
      max_score: 11,
      seed: 0xc0ffee,
      tick_hz: 60,
      ball_radius: 0.02,
      paddle_width: 0.02,
    };

    const wasmGame = new (globalThis as any).WasmGame(JSON.stringify(config));
    const gameAdapter = new WasmGameAdapter(wasmGame);

    // Create lockstep instance
    lockstep = new Lockstep(
      gameAdapter,
      currentTransport,
      60, // tick_hz
      localSide,
      localSide === Side.Left // Left side is timekeeper
    );

    lockstep.start();
    networkingMode = localSide === Side.Left ? "host" : "join";

    // Switch to game screen
    screenManager.showScreen(Screen.Game);

    // Start networked game loop
    startNetworkedGameLoop();

    console.log(
      `Networked game started as ${
        localSide === Side.Left ? "host (left)" : "join (right)"
      }`
    );
  } catch (error) {
    console.error("Failed to start networked game:", error);
    const errorMsg = `Failed to start game: ${error}`;

    if (networkingMode === "host") {
      updateHostScreen("", errorMsg);
    } else {
      updateJoinScreen(errorMsg);
    }
  }
}

/**
 * Start the networked game loop
 */
function startNetworkedGameLoop(): void {
  let lastTickTime = 0;
  const tickInterval = 1000 / 60; // 60 FPS

  function networkGameLoop(currentTime: number): void {
    if (!lockstep) {
      return;
    }

    // Submit local input if enough time has passed
    if (currentTime - lastTickTime >= tickInterval) {
      const rawInput = inputManager.getInputState();
      // For networked games, use left paddle input as local player
      lockstep.onLocalInput(rawInput.leftPaddleAxis, rawInput.leftButtons);

      lastTickTime = currentTime;
    }

    // Try to advance simulation
    const { view, events } = lockstep.tick();

    // Render the game
    gameRenderer.updateStatus(view);
    gameRenderer.renderGame(view);

    // Handle lockstep events
    for (const event of events) {
      if (event.type === "game_advanced") {
        console.log("Game advanced to tick", event.tick);
      }
    }

    // Check for disconnection
    if (!lockstep.isConnected()) {
      console.warn("Peer disconnected");
      cleanupNetworking();
      gameStateManager.stopGame();
      return;
    }

    // Continue the loop only if we're still in a networked game
    if (
      networkingMode !== "local" &&
      screenManager.currentScreen === Screen.Game
    ) {
      requestAnimationFrame(networkGameLoop);
    }
  }

  requestAnimationFrame(networkGameLoop);
}

/**
 * Cleanup networking resources
 */
function cleanupNetworking(): void {
  if (lockstep) {
    lockstep.stop();
    lockstep = null;
  }

  if (currentTransport) {
    try {
      currentTransport.close();
    } catch (error) {
      console.warn("Error closing transport:", error);
    }
    currentTransport = null;
  }

  networkingMode = "local";
}

/**
 * Start the rendering loop (for local games only)
 */
function startRenderLoop(): void {
  let lastRenderTime = 0;

  function renderLoop(currentTime: number): void {
    // Only render local games in this loop - networked games have their own loop
    if (networkingMode === "local") {
      // Render at 60 FPS
      if (currentTime - lastRenderTime >= 16.67) {
        const view = gameStateManager.getCurrentView();

        if (view && gameStateManager.isGameRunning()) {
          gameRenderer.updateStatus(view);
          gameRenderer.renderGame(view);

          // Update FAB visibility based on game state
          const status =
            typeof view.status === "string" ? view.status : "Playing";
          inputManager.updateFABVisibility(status);
        }

        lastRenderTime = currentTime;
      }
    }

    requestAnimationFrame(renderLoop);
  }

  requestAnimationFrame(renderLoop);
  console.log("Render loop started");
}

/**
 * Initialize host mode with PeerJS
 */
async function initializeHostMode(): Promise<void> {
  try {
    console.log("Initializing host mode with PeerJS...");
    updateHostScreen("", "Creating peer connection...");
    screenManager.showScreen(Screen.Host);

    // Create host transport and get peer ID
    const { transport, peerId } = await PeerTransportFactory.createHost();

    console.log("PeerJS host created with ID:", peerId);

    currentTransport = transport;
    networkingMode = "host";

    updateHostScreen(peerId, "Waiting for peer to connect...");

    // Setup QR code display for the peer ID
    setupHostQRDisplay(peerId, "host-peer-id-section");

    // Monitor for incoming connections
    const checkConnection = () => {
      if (transport.isOpen()) {
        updateHostScreen(peerId, "Connected! Starting game...");
        startNetworkedGame(Side.Left); // Host is left side
      } else {
        setTimeout(checkConnection, 500);
      }
    };

    setTimeout(checkConnection, 1000);
  } catch (error) {
    console.error("Failed to initialize host mode:", error);
    const errorMessage = error instanceof Error ? error.message : String(error);
    updateHostScreen("", `Failed to create host: ${errorMessage}`);
    screenManager.showScreen(Screen.Host);
  }
}

/**
 * Initialize join mode and prepare for peer ID input
 */
function initializeJoinMode(): void {
  networkingMode = "join";
  updateJoinScreen("Enter the host's Game ID to connect");
  screenManager.showScreen(Screen.Join);

  // Setup QR scanning for guest screen
  setupGuestQRScanning("join-peer-id-input", "join-peer-id-section");
}

// Start the application
main().catch(console.error);
