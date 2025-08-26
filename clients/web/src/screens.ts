// Screen management and mode selection for Pong Web Client

export enum GameMode {
  VsLocal2 = "VsLocal2",
  VsAI = "VsAI",
  VsWall = "VsWall",
}

export enum Screen {
  ModeSelect = "ModeSelect",
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
    const gameScreen = document.getElementById("game-screen");
    const gameOverScreen = document.getElementById("game-over-screen");

    if (!modeSelectScreen || !gameScreen || !gameOverScreen) {
      console.error("Screen elements not found");
      return;
    }

    // Hide all screens
    modeSelectScreen.style.display = "none";
    gameScreen.style.display = "none";
    gameOverScreen.style.display = "none";

    // Show current screen
    switch (this.currentScreen) {
      case Screen.ModeSelect:
        modeSelectScreen.style.display = "block";
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
  const vsLocal2Btn = document.getElementById("btn-vs-local2");
  const vsAIBtn = document.getElementById("btn-vs-ai");
  const vsWallBtn = document.getElementById("btn-vs-wall");

  if (!vsLocal2Btn || !vsAIBtn || !vsWallBtn) {
    console.error("Mode selection buttons not found");
    return;
  }

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
