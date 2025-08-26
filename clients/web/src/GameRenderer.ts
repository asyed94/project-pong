// Game rendering module - handles all visual output

import type { GameView } from "./types.js";
import { FIELD_DIMENSIONS, UNICODE_CHARS, GAME_CONFIG } from "./constants.js";

export class GameRenderer {
  private fieldElement: HTMLElement | null = null;
  private gameInfoElement: HTMLElement | null = null;

  constructor() {
    this.fieldElement = document.getElementById("field");
    this.gameInfoElement = document.getElementById("game-info");
  }

  /**
   * Update status display based on current game state - adapts to mobile
   */
  updateStatus(view: GameView): void {
    if (!this.gameInfoElement) return;

    const isLandscapeMobile = this.isLandscapeMobile();
    const statusText = this.getStatusText(
      view,
      isLandscapeMobile || this.isPortraitMobile()
    );

    let infoText = "";
    if (isLandscapeMobile) {
      // Compact format for landscape mobile
      infoText = `Score: ${view.score[0]}-${view.score[1]} • ${statusText} • T:${view.tick}`;
    } else {
      // Full format for desktop and portrait
      infoText = `Score: ${view.score[0]} - ${view.score[1]}    Status: ${statusText}    Tick: ${view.tick}`;
    }

    this.gameInfoElement.textContent = infoText;
  }

  /**
   * Render the game field - adapts to screen size
   */
  renderGame(view: GameView): void {
    if (!this.fieldElement) return;

    try {
      const { width: fieldWidth, height: fieldHeight } =
        this.getOptimalFieldSize();
      const field: string[][] = [];

      // Initialize field with spaces
      for (let y = 0; y < fieldHeight; y++) {
        field[y] = [];
        for (let x = 0; x < fieldWidth; x++) {
          field[y][x] = " ";
        }
      }

      // Convert fixed-point coordinates to normalized coordinates
      const normalizedBallX = view.ball_pos.x / GAME_CONFIG.FIXED_POINT_SCALE;
      const normalizedBallY = view.ball_pos.y / GAME_CONFIG.FIXED_POINT_SCALE;
      const normalizedLeftPaddleY =
        view.left_paddle_y / GAME_CONFIG.FIXED_POINT_SCALE;
      const normalizedRightPaddleY =
        view.right_paddle_y / GAME_CONFIG.FIXED_POINT_SCALE;
      const normalizedPaddleXOffset =
        view.paddle_x_offset / GAME_CONFIG.FIXED_POINT_SCALE;
      const normalizedPaddleHalfH =
        view.paddle_half_h / GAME_CONFIG.FIXED_POINT_SCALE;

      // Convert normalized coordinates to field coordinates
      const ballX = Math.round(normalizedBallX * (fieldWidth - 1));
      const ballY = Math.round((1 - normalizedBallY) * (fieldHeight - 1));

      // Calculate paddle positions
      const leftPaddleX = Math.round(normalizedPaddleXOffset * fieldWidth);
      const rightPaddleX =
        fieldWidth - 1 - Math.round(normalizedPaddleXOffset * fieldWidth);

      const leftPaddleY = Math.round(
        (1 - normalizedLeftPaddleY) * (fieldHeight - 1)
      );
      const rightPaddleY = Math.round(
        (1 - normalizedRightPaddleY) * (fieldHeight - 1)
      );

      // Calculate paddle height
      const paddleHeight = Math.max(
        1,
        Math.round(normalizedPaddleHalfH * 2 * fieldHeight)
      );

      // Place ball
      if (
        ballX >= 0 &&
        ballX < fieldWidth &&
        ballY >= 0 &&
        ballY < fieldHeight
      ) {
        field[ballY][ballX] = UNICODE_CHARS.BALL;
      }

      // Place center line
      const centerX = Math.floor(fieldWidth / 2);
      for (let y = 0; y < fieldHeight; y++) {
        if (field[y][centerX] === " ") {
          field[y][centerX] = UNICODE_CHARS.CENTER_LINE;
        }
      }

      // Place left paddle
      const leftPaddleStart = Math.max(
        0,
        leftPaddleY - Math.floor(paddleHeight / 2)
      );
      const leftPaddleEnd = Math.min(
        fieldHeight - 1,
        leftPaddleStart + paddleHeight
      );

      for (let y = leftPaddleStart; y <= leftPaddleEnd; y++) {
        if (leftPaddleX >= 0 && leftPaddleX < fieldWidth) {
          field[y][leftPaddleX] = UNICODE_CHARS.PADDLE;
        }
      }

      // Place right paddle
      const rightPaddleStart = Math.max(
        0,
        rightPaddleY - Math.floor(paddleHeight / 2)
      );
      const rightPaddleEnd = Math.min(
        fieldHeight - 1,
        rightPaddleStart + paddleHeight
      );

      for (let y = rightPaddleStart; y <= rightPaddleEnd; y++) {
        if (rightPaddleX >= 0 && rightPaddleX < fieldWidth) {
          field[y][rightPaddleX] = UNICODE_CHARS.PADDLE;
        }
      }

      // Convert field to string and add borders
      const fieldContent = field.map((row) => row.join(""));
      const output = this.addBorders(fieldContent, fieldWidth);

      this.fieldElement.textContent = output;
    } catch (error) {
      console.error("Error rendering game:", error);
      this.fieldElement.textContent = "Error rendering game";
    }
  }

  /**
   * Get optimal field dimensions based on screen orientation and size
   */
  private getOptimalFieldSize(): { width: number; height: number } {
    if (this.isLandscapeMobile()) {
      return FIELD_DIMENSIONS.MOBILE_LANDSCAPE;
    } else if (this.isPortraitMobile()) {
      return FIELD_DIMENSIONS.MOBILE_PORTRAIT;
    }
    return FIELD_DIMENSIONS.DESKTOP;
  }

  /**
   * Get simplified status text for mobile devices
   */
  private getStatusText(view: GameView, isMobile: boolean): string {
    if (typeof view.status === "string") {
      switch (view.status) {
        case "Lobby":
          return isMobile
            ? "Waiting"
            : "Waiting for players to be ready (SPACE)";
        case "Playing":
          return "Playing";
        default:
          return view.status;
      }
    } else if (typeof view.status === "object") {
      if ("Countdown" in view.status) {
        return "Get ready...";
      } else if ("Scored" in view.status) {
        return "Point scored!";
      } else if ("GameOver" in view.status) {
        return "Game Over!";
      }
    }
    return "Unknown";
  }

  /**
   * Add unicode borders around field content
   */
  private addBorders(fieldContent: string[], fieldWidth: number): string {
    const topBorder =
      UNICODE_CHARS.BORDER.TOP_LEFT +
      UNICODE_CHARS.BORDER.HORIZONTAL.repeat(fieldWidth) +
      UNICODE_CHARS.BORDER.TOP_RIGHT;

    const bottomBorder =
      UNICODE_CHARS.BORDER.BOTTOM_LEFT +
      UNICODE_CHARS.BORDER.HORIZONTAL.repeat(fieldWidth) +
      UNICODE_CHARS.BORDER.BOTTOM_RIGHT;

    const lines = [];
    lines.push(topBorder);
    for (const row of fieldContent) {
      lines.push(
        UNICODE_CHARS.BORDER.VERTICAL + row + UNICODE_CHARS.BORDER.VERTICAL
      );
    }
    lines.push(bottomBorder);

    return lines.join("\n");
  }

  /**
   * Check if device is in landscape mobile mode
   */
  private isLandscapeMobile(): boolean {
    return window.matchMedia("(orientation: landscape) and (max-height: 500px)")
      .matches;
  }

  /**
   * Check if device is in portrait mobile mode
   */
  private isPortraitMobile(): boolean {
    return window.matchMedia("(max-width: 768px) and (orientation: portrait)")
      .matches;
  }

  /**
   * Update field and info elements (useful for dynamic DOM changes)
   */
  updateElements(): void {
    this.fieldElement = document.getElementById("field");
    this.gameInfoElement = document.getElementById("game-info");
  }
}
