// Main entry point for Pong Web Client

import { GameStateManager } from "./GameStateManager.js";
import { GameRenderer } from "./GameRenderer.js";
import { InputManager } from "./InputManager.js";
import { MobileController } from "./MobileController.js";
import {
  initializeModeSelection,
  initializeGameOverScreen,
} from "./screens.js";

// Global instances
let gameStateManager: GameStateManager;
let gameRenderer: GameRenderer;
let inputManager: InputManager;
let mobileController: MobileController;

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
  // Initialize mode selection
  initializeModeSelection((mode) => {
    gameStateManager.updateControlInstructions(mode);
    gameStateManager.startGame(mode);
  });

  // Initialize game over screen
  initializeGameOverScreen(
    () => {
      // Rematch
      gameStateManager.rematch();
    },
    () => {
      // Back to menu
      gameStateManager.stopGame();
    }
  );

  // Back to menu button in game screen
  const backToMenuBtn = document.getElementById("back-to-menu-btn");
  if (backToMenuBtn) {
    backToMenuBtn.addEventListener("click", () => {
      gameStateManager.stopGame();
    });
  }

  // Handle ESC key for menu
  document.addEventListener("keydown", (event) => {
    if (event.code === "Escape" && gameStateManager.isGameRunning()) {
      gameStateManager.stopGame();
    }
  });
}

/**
 * Start the rendering loop
 */
function startRenderLoop(): void {
  let lastRenderTime = 0;

  function renderLoop(currentTime: number): void {
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

    requestAnimationFrame(renderLoop);
  }

  requestAnimationFrame(renderLoop);
  console.log("Render loop started");
}

// Start the application
main().catch(console.error);
