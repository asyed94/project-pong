// Game state management module - handles game loop, WASM integration, and game logic

import type { WasmGame, GameView, GameEvent } from "./types.js";
import {
  GameMode,
  Screen,
  screenManager,
  updateGameOverScreen,
} from "./screens.js";
import { AIController, AI_DIFFICULTY_PRESETS } from "./ai.js";
import { GAME_CONFIG } from "./constants.js";
import type { InputState } from "./InputManager.js";

export class GameStateManager {
  private wasmModule: any = null;
  private wasmGame: WasmGame | null = null;
  private currentGameMode: GameMode | null = null;
  private aiController: AIController | null = null;
  private gameLoopRunning = false;
  private tickCounter = 0;

  /**
   * Initialize the WASM module and create game instance
   */
  async initialize(): Promise<void> {
    try {
      console.log("Loading WASM module...");

      // Import the WASM module
      this.wasmModule = await import("../wasm/pong_core.js");
      await this.wasmModule.default();

      console.log("WASM module loaded successfully");

      // Create game instance with default config
      this.wasmGame = new this.wasmModule.WasmGame(
        JSON.stringify(GAME_CONFIG.DEFAULT_GAME_CONFIG)
      );
      console.log("Game instance created");

      console.log("WASM initialized, waiting for mode selection");
    } catch (error) {
      console.error("Failed to initialize WASM:", error);
      throw new Error(
        `WASM initialization failed: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
    }
  }

  /**
   * Start a new game with the selected mode
   */
  startGame(mode: GameMode): void {
    if (!this.wasmGame) {
      console.error("Cannot start game - WASM not initialized");
      return;
    }

    this.currentGameMode = mode;

    // Reset game state
    this.wasmGame.reset_match();
    console.log("Game state reset for new match");

    // Initialize AI if needed
    if (mode === GameMode.VsAI) {
      this.aiController = new AIController(AI_DIFFICULTY_PRESETS.medium);
    } else {
      this.aiController = null;
    }

    // Switch to game screen
    screenManager.showScreen(Screen.Game);

    // Start the game loop
    this.startGameLoop();

    console.log(`Started game in ${mode} mode`);
  }

  /**
   * Stop current game and return to menu
   */
  stopGame(): void {
    this.gameLoopRunning = false;
    this.currentGameMode = null;
    this.aiController = null;
    this.tickCounter = 0;
    screenManager.showScreen(Screen.ModeSelect);
  }

  /**
   * Get current game view (for rendering)
   */
  getCurrentView(): GameView | null {
    if (!this.wasmGame) return null;

    try {
      const viewJson = this.wasmGame.view_json();
      return JSON.parse(viewJson);
    } catch (error) {
      console.error("Failed to get game view:", error);
      return null;
    }
  }

  /**
   * Check if game is currently running
   */
  isGameRunning(): boolean {
    return this.gameLoopRunning;
  }

  /**
   * Get current game mode
   */
  getCurrentMode(): GameMode | null {
    return this.currentGameMode;
  }

  /**
   * Update control instructions based on game mode
   */
  updateControlInstructions(mode: GameMode): void {
    const controlTextElement = document.getElementById("control-text");
    if (!controlTextElement) return;

    let controlText = "";

    switch (mode) {
      case GameMode.VsLocal2:
        controlText =
          "P1: W/S (up/down)  P2: ↑/↓ (up/down)  SPACE: Ready  ESC: Menu";
        break;
      case GameMode.VsAI:
        controlText = "Player: W/S (up/down)  SPACE: Ready  ESC: Menu";
        break;
      case GameMode.VsWall:
        controlText = "Player: W/S (up/down)  SPACE: Ready  ESC: Menu";
        break;
    }

    controlTextElement.textContent = controlText;
  }

  /**
   * Enhanced game loop with mode support
   */
  private startGameLoop(): void {
    if (this.gameLoopRunning || !this.wasmGame) return;

    this.gameLoopRunning = true;
    let lastTime = 0;
    this.tickCounter = 0;

    const gameLoop = (currentTime: number): void => {
      if (!this.wasmGame || !this.gameLoopRunning) return;

      // Run at 60 FPS
      if (currentTime - lastTime >= GAME_CONFIG.FRAME_RATE) {
        try {
          // Get input state from the input manager (will be injected)
          const inputState = this.getCurrentInputState(currentTime);

          // Step the game with processed input state
          const event = this.wasmGame.step(
            this.tickCounter,
            inputState.leftPaddleAxis,
            inputState.leftButtons,
            inputState.rightPaddleAxis,
            inputState.rightButtons
          );

          // Handle game events
          if (event) {
            const gameEvent: GameEvent = JSON.parse(event);
            this.handleGameEvent(gameEvent);
          }

          // Check for game over condition
          this.checkGameOverCondition();

          this.tickCounter++;
          lastTime = currentTime;
        } catch (error) {
          console.error("Error in game loop:", error);
          return;
        }
      }

      requestAnimationFrame(gameLoop);
    };

    requestAnimationFrame(gameLoop);
    console.log("Game loop started");
  }

  /**
   * Get current input state (to be overridden by dependency injection)
   */
  private getCurrentInputState(_currentTime: number): InputState {
    // This is a placeholder - the actual implementation will inject the input manager
    console.warn("getCurrentInputState not properly injected");
    return {
      leftPaddleAxis: 0,
      rightPaddleAxis: 0,
      leftButtons: 0,
      rightButtons: 0,
    };
  }

  /**
   * Set input state provider (dependency injection)
   */
  setInputStateProvider(provider: (currentTime: number) => InputState): void {
    this.getCurrentInputState = provider;
  }

  /**
   * Calculate input based on current game mode
   */
  processInputForGameMode(
    rawInput: InputState,
    currentTime: number
  ): InputState {
    const processedInput: InputState = { ...rawInput };

    if (!this.currentGameMode || !this.wasmGame) return processedInput;

    switch (this.currentGameMode) {
      case GameMode.VsLocal2:
        // Two players - use input as is
        break;

      case GameMode.VsAI:
        // AI controls right paddle
        if (this.aiController) {
          try {
            const view = this.getCurrentView();
            if (view) {
              // Calculate AI input
              const aiInput = this.aiController.generateInput(
                view.ball_pos.x,
                view.ball_pos.y,
                0, // Ball velocity not directly available in view
                0, // Ball velocity not directly available in view
                view.right_paddle_y,
                currentTime
              );

              processedInput.rightPaddleAxis = aiInput;

              // AI always ready in lobby
              if (typeof view.status === "string" && view.status === "Lobby") {
                processedInput.rightButtons |= 1; // AI is always ready
              }
            }
          } catch (error) {
            console.error("AI calculation error:", error);
          }
        }
        break;

      case GameMode.VsWall:
        // Wall mode - no right paddle, only left player
        processedInput.rightPaddleAxis = 0;
        processedInput.rightButtons = processedInput.leftButtons; // Mirror ready state
        break;
    }

    return processedInput;
  }

  /**
   * Handle game events
   */
  private handleGameEvent(event: GameEvent): void {
    console.log("Game event:", event);
    // Handle specific events based on type
    // This would be expanded based on GameEvent structure
  }

  /**
   * Check for game over condition and handle transitions
   */
  private checkGameOverCondition(): void {
    if (!this.wasmGame) return;

    try {
      const view = this.getCurrentView();
      if (!view) return;

      // Check if game is over
      if (typeof view.status === "object" && "GameOver" in view.status) {
        this.handleGameOver(view);
      }
    } catch (error) {
      console.error("Error checking game over:", error);
    }
  }

  /**
   * Handle game over transition
   */
  private handleGameOver(view: GameView): void {
    // Stop the game loop
    this.gameLoopRunning = false;

    // Determine winner
    const [leftScore, rightScore] = view.score;
    let winner = "Game Over";

    if (this.currentGameMode === GameMode.VsLocal2) {
      winner = leftScore > rightScore ? "Player 1 Wins!" : "Player 2 Wins!";
    } else if (this.currentGameMode === GameMode.VsAI) {
      winner = leftScore > rightScore ? "You Win!" : "AI Wins!";
    } else if (this.currentGameMode === GameMode.VsWall) {
      winner = `Final Score: ${leftScore}`;
    }

    // Update game over screen
    updateGameOverScreen(winner, view.score);

    // Switch to game over screen
    screenManager.showScreen(Screen.GameOver);

    console.log("Game over:", winner);
  }

  /**
   * Start a rematch with the same mode
   */
  rematch(): void {
    if (this.currentGameMode) {
      this.startGame(this.currentGameMode);
    }
  }

  /**
   * Get the current game instance (for debugging)
   */
  getGame(): WasmGame | null {
    return this.wasmGame;
  }

  /**
   * Get the WASM module (for debugging)
   */
  getWasmModule(): any {
    return this.wasmModule;
  }

  /**
   * Reset AI controller (useful for difficulty changes)
   */
  resetAI(): void {
    if (this.aiController) {
      this.aiController.reset();
    }
  }
}
