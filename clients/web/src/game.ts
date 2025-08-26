// Legacy game module - functionality moved to modular architecture
// This file provides backwards compatibility for any existing code that imports from game.js

import { GameStateManager } from "./GameStateManager.js";
import { InputManager } from "./InputManager.js";
import type { WasmGame } from "./types.js";
import { GameMode } from "./screens.js";

// Legacy compatibility - these functions are now handled by the modular architecture

/**
 * @deprecated Use GameStateManager.initialize() instead
 */
export async function initializeGame(): Promise<void> {
  console.warn(
    "initializeGame() is deprecated. Use GameStateManager.initialize() instead."
  );
  const gameStateManager = new GameStateManager();
  await gameStateManager.initialize();
}

/**
 * @deprecated Use InputManager.initialize() instead
 */
export function initializeInputSystems(): void {
  console.warn(
    "initializeInputSystems() is deprecated. Use InputManager.initialize() instead."
  );
  const inputManager = new InputManager();
  inputManager.initialize();
}

/**
 * @deprecated Use the new modular architecture in main.ts instead
 */
export function initializeScreenManagement(): void {
  console.warn(
    "initializeScreenManagement() is deprecated. Use the new modular architecture instead."
  );
  // This function is now handled in main.ts
}

/**
 * @deprecated Use GameStateManager.startGame() instead
 */
export function startGame(_mode: GameMode): void {
  console.warn(
    "startGame() is deprecated. Use GameStateManager.startGame() instead."
  );
  // Legacy function - functionality moved to GameStateManager
}

/**
 * @deprecated Use GameStateManager.stopGameLoop() instead
 */
export function stopGameLoop(): void {
  console.warn(
    "stopGameLoop() is deprecated. Use GameStateManager.stopGameLoop() instead."
  );
  // Legacy function - functionality moved to GameStateManager
}

/**
 * @deprecated Use GameStateManager.getGame() instead
 */
export function getGame(): WasmGame | null {
  console.warn(
    "getGame() is deprecated. Use GameStateManager.getGame() instead."
  );
  return null;
}

/**
 * @deprecated Use GameStateManager.getWasmModule() instead
 */
export function getWasmModule(): any {
  console.warn(
    "getWasmModule() is deprecated. Use GameStateManager.getWasmModule() instead."
  );
  return null;
}

// Legacy exports for backwards compatibility
export { GameMode } from "./screens.js";
export type { WasmGame, GameView, GameEvent } from "./types.js";
