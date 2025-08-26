// AI logic and physics for different game modes

import { INPUT_CONFIG } from "./constants.js";

export interface AISettings {
  difficulty: number; // 0.0 to 1.0, where 1.0 is perfect
  reactionDelay: number; // milliseconds of delay
  errorRate: number; // 0.0 to 1.0, chance of making mistakes
}

export const AI_DIFFICULTY_PRESETS: { [key: string]: AISettings } = {
  easy: {
    difficulty: 0.3,
    reactionDelay: 200,
    errorRate: 0.15,
  },
  medium: {
    difficulty: 0.6,
    reactionDelay: 100,
    errorRate: 0.08,
  },
  hard: {
    difficulty: 0.8,
    reactionDelay: 50,
    errorRate: 0.03,
  },
  expert: {
    difficulty: 0.95,
    reactionDelay: 20,
    errorRate: 0.01,
  },
} as const;

/**
 * AI Controller for VsAI mode
 */
export class AIController {
  private settings: AISettings;
  private lastUpdate = 0;
  private targetInput = 0;
  private currentInput = 0;
  private randomSeed: number;

  constructor(settings: AISettings = AI_DIFFICULTY_PRESETS.medium) {
    this.settings = settings;
    this.randomSeed = Date.now() % 233280; // Initialize with bounded seed
  }

  /**
   * Generate AI input based on ball and paddle positions
   */
  generateInput(
    ballX: number,
    ballY: number,
    ballVelX: number,
    ballVelY: number,
    paddleY: number,
    currentTime: number
  ): number {
    const deltaTime = currentTime - this.lastUpdate;
    this.lastUpdate = currentTime;

    // Only react if ball is moving towards AI paddle (right side)
    const shouldReact = ballVelX > 0;

    if (!shouldReact) {
      // Return to center position when ball is moving away
      const centerY = 0.5;
      const diffFromCenter = centerY - paddleY;
      this.targetInput =
        Math.sign(diffFromCenter) *
        Math.min(Math.abs(diffFromCenter) * 100, INPUT_CONFIG.AXIS_RANGE.MAX);
    } else {
      // Predict where ball will be when it reaches paddle
      const timeToReachPaddle = this.estimateTimeToReachPaddle(ballX, ballVelX);
      const predictedBallY = ballY + ballVelY * timeToReachPaddle;

      // Calculate desired paddle movement
      const diff = predictedBallY - paddleY;
      const strength = this.settings.difficulty;

      // Apply difficulty scaling
      let desiredInput = diff * strength * 200; // Scale factor for responsiveness

      // Add some error based on AI settings
      if (this.settings.errorRate > 0) {
        const errorAmount = this.settings.errorRate * 50 * this.pseudoRandom();
        desiredInput += errorAmount;
      }

      // Apply reaction delay
      if (deltaTime >= this.settings.reactionDelay) {
        this.targetInput = Math.max(
          INPUT_CONFIG.AXIS_RANGE.MIN,
          Math.min(INPUT_CONFIG.AXIS_RANGE.MAX, desiredInput)
        );
      }
    }

    // Smooth movement towards target input (prevents jerky movement)
    this.currentInput +=
      (this.targetInput - this.currentInput) * INPUT_CONFIG.INPUT_SMOOTHING;

    return Math.round(
      Math.max(
        INPUT_CONFIG.AXIS_RANGE.MIN,
        Math.min(INPUT_CONFIG.AXIS_RANGE.MAX, this.currentInput)
      )
    );
  }

  /**
   * Estimate time for ball to reach AI paddle (simplified physics)
   */
  private estimateTimeToReachPaddle(ballX: number, ballVelX: number): number {
    if (ballVelX <= 0) return Infinity;

    // Assume paddle is at x = 0.9 (near right edge)
    const paddleX = 0.9;
    const distance = paddleX - ballX;

    return Math.max(0, distance / ballVelX);
  }

  /**
   * Simple pseudo-random number generator for consistent AI behavior
   */
  private pseudoRandom(): number {
    this.randomSeed = (this.randomSeed * 9301 + 49297) % 233280;
    return (this.randomSeed / 233280) * 2 - 1; // Returns value between -1 and 1
  }

  /**
   * Update AI difficulty
   */
  setDifficulty(settings: AISettings): void {
    this.settings = settings;
  }

  /**
   * Reset AI state
   */
  reset(): void {
    this.currentInput = INPUT_CONFIG.AXIS_RANGE.NEUTRAL;
    this.targetInput = INPUT_CONFIG.AXIS_RANGE.NEUTRAL;
    this.lastUpdate = 0;
    this.randomSeed = Date.now() % 233280;
  }
}

/**
 * Wall physics implementation for VsWall mode
 * This modifies the game logic to make the right edge act as a perfect reflector
 */
export class WallPhysics {
  /**
   * Check if ball should bounce off the wall (right edge)
   */
  static shouldBounceOffWall(ballX: number, ballVelX: number): boolean {
    // Ball is hitting the right edge and moving right
    return ballX >= 0.98 && ballVelX > 0;
  }

  /**
   * Calculate reflected velocity when ball hits wall
   */
  static reflectOffWall(
    ballVelX: number,
    ballVelY: number
  ): { velX: number; velY: number } {
    return {
      velX: -ballVelX, // Perfect reflection, reverse X velocity
      velY: ballVelY, // Keep Y velocity unchanged
    };
  }

  /**
   * Adjust ball position after wall collision
   */
  static adjustBallPosition(ballX: number): number {
    // Ensure ball stays within bounds after reflection
    return Math.min(0.98, ballX);
  }
}
