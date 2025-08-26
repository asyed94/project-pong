// Screen management and mode selection for Pong Web Client

import { PeerIdUtils } from "./peer_transport.js";

export enum GameMode {
  VsLocal2 = "VsLocal2",
  VsAI = "VsAI",
  VsWall = "VsWall",
  Host = "Host",
  Join = "Join",
}

export enum Screen {
  ModeSelect = "ModeSelect",
  Host = "Host",
  Join = "Join",
  Game = "Game",
  GameOver = "GameOver",
}

export interface ScreenManager {
  currentScreen: Screen;
  currentMode: GameMode | null;
  showScreen(screen: Screen): void;
  setMode(mode: GameMode): void;
}

/**
 * Screen manager implementation
 */
class ScreenManagerImpl implements ScreenManager {
  currentScreen: Screen = Screen.ModeSelect;
  currentMode: GameMode | null = null;

  showScreen(screen: Screen): void {
    this.currentScreen = screen;
    this.updateDOM();
  }

  setMode(mode: GameMode): void {
    this.currentMode = mode;
  }

  private updateDOM(): void {
    const modeSelectScreen = document.getElementById("mode-select-screen");
    const hostScreen = document.getElementById("host-screen");
    const joinScreen = document.getElementById("join-screen");
    const gameScreen = document.getElementById("game-screen");
    const gameOverScreen = document.getElementById("game-over-screen");

    if (
      !modeSelectScreen ||
      !hostScreen ||
      !joinScreen ||
      !gameScreen ||
      !gameOverScreen
    ) {
      console.error("Screen elements not found");
      return;
    }

    // Hide all screens
    modeSelectScreen.style.display = "none";
    hostScreen.style.display = "none";
    joinScreen.style.display = "none";
    gameScreen.style.display = "none";
    gameOverScreen.style.display = "none";

    // Show current screen
    switch (this.currentScreen) {
      case Screen.ModeSelect:
        modeSelectScreen.style.display = "block";
        break;
      case Screen.Host:
        hostScreen.style.display = "block";
        break;
      case Screen.Join:
        joinScreen.style.display = "block";
        break;
      case Screen.Game:
        gameScreen.style.display = "block";
        break;
      case Screen.GameOver:
        gameOverScreen.style.display = "block";
        break;
    }
  }
}

// Export singleton instance
export const screenManager: ScreenManager = new ScreenManagerImpl();

/**
 * Initialize mode selection screen
 */
export function initializeModeSelection(
  onModeSelected: (mode: GameMode) => void
): void {
  const hostBtn = document.getElementById("btn-host");
  const joinBtn = document.getElementById("btn-join");
  const vsLocal2Btn = document.getElementById("btn-vs-local2");
  const vsAIBtn = document.getElementById("btn-vs-ai");
  const vsWallBtn = document.getElementById("btn-vs-wall");

  if (!hostBtn || !joinBtn || !vsLocal2Btn || !vsAIBtn || !vsWallBtn) {
    console.error("Mode selection buttons not found");
    return;
  }

  hostBtn.addEventListener("click", () => {
    screenManager.setMode(GameMode.Host);
    onModeSelected(GameMode.Host); // This will trigger the actual host initialization
  });

  joinBtn.addEventListener("click", () => {
    screenManager.setMode(GameMode.Join);
    onModeSelected(GameMode.Join); // This will trigger the actual join initialization
  });

  vsLocal2Btn.addEventListener("click", () => {
    screenManager.setMode(GameMode.VsLocal2);
    onModeSelected(GameMode.VsLocal2);
  });

  vsAIBtn.addEventListener("click", () => {
    screenManager.setMode(GameMode.VsAI);
    onModeSelected(GameMode.VsAI);
  });

  vsWallBtn.addEventListener("click", () => {
    screenManager.setMode(GameMode.VsWall);
    onModeSelected(GameMode.VsWall);
  });

  console.log("Mode selection initialized");
}

/**
 * Initialize game over screen
 */
export function initializeGameOverScreen(
  onRematch: () => void,
  onBackToMenu: () => void
): void {
  const rematchBtn = document.getElementById("btn-rematch");
  const menuBtn = document.getElementById("btn-back-to-menu");

  if (!rematchBtn || !menuBtn) {
    console.error("Game over buttons not found");
    return;
  }

  rematchBtn.addEventListener("click", onRematch);
  menuBtn.addEventListener("click", onBackToMenu);

  console.log("Game over screen initialized");
}

/**
 * Update game over screen with results
 */
export function updateGameOverScreen(
  winner: string,
  score: [number, number]
): void {
  const winnerText = document.getElementById("winner-text");
  const finalScore = document.getElementById("final-score");

  if (winnerText) {
    winnerText.textContent = winner;
  }

  if (finalScore) {
    finalScore.textContent = `Final Score: ${score[0]} - ${score[1]}`;
  }
}

/**
 * Initialize host screen for PeerJS connection
 */
export function initializeHostScreen(onBackToMenu: () => void): void {
  const backBtn = document.getElementById("btn-host-back");

  if (!backBtn) {
    console.error("Host screen back button not found");
    return;
  }

  backBtn.addEventListener("click", () => {
    screenManager.showScreen(Screen.ModeSelect);
    onBackToMenu();
  });

  console.log("Host screen initialized");
}

/**
 * Initialize join screen for PeerJS connection
 */
export function initializeJoinScreen(
  onJoinGame: (hostPeerId: string) => void,
  onBackToMenu: () => void
): void {
  const joinBtn = document.getElementById("btn-join-connect");
  const backBtn = document.getElementById("btn-join-back");
  const peerIdInput = document.getElementById(
    "join-peer-id-input"
  ) as HTMLInputElement;

  if (!joinBtn || !backBtn || !peerIdInput) {
    console.error("Join screen elements not found");
    return;
  }

  joinBtn.addEventListener("click", () => {
    const hostPeerId = PeerIdUtils.cleanPeerId(peerIdInput.value.trim());
    if (hostPeerId && PeerIdUtils.isValidPeerId(hostPeerId)) {
      onJoinGame(hostPeerId);
    } else {
      alert("Please enter a valid Game ID from the host");
    }
  });

  backBtn.addEventListener("click", () => {
    screenManager.showScreen(Screen.ModeSelect);
    onBackToMenu();
  });

  console.log("Join screen initialized");
}

/**
 * Update host screen with peer ID and status
 */
export function updateHostScreen(peerId: string, status: string): void {
  const peerIdOutput = document.getElementById(
    "host-peer-id-output"
  ) as HTMLInputElement;
  const statusElement = document.getElementById("host-status");

  if (peerIdOutput) {
    peerIdOutput.value = PeerIdUtils.formatPeerId(peerId);
  }

  if (statusElement) {
    statusElement.textContent = status;
  }

  // Show formatted connection string
  const connectionString = document.getElementById("host-connection-string");
  if (connectionString && peerId) {
    connectionString.textContent = PeerIdUtils.createConnectionString(peerId);
  }
}

/**
 * Update join screen with status
 */
export function updateJoinScreen(status: string): void {
  const statusElement = document.getElementById("join-status");

  if (statusElement) {
    statusElement.textContent = status;
  }
}
