// Input management module - handles keyboard, touch, and other input systems

import {
  INPUT_CONFIG,
  MOBILE_CONFIG,
  STORAGE_KEYS,
  VIEWPORT_CONFIG,
  FAST_INPUT_CONFIG,
} from "./constants.js";

export interface InputState {
  leftPaddleAxis: number; // [-127, 127]
  rightPaddleAxis: number; // [-127, 127]
  leftButtons: number; // Bitfield
  rightButtons: number; // Bitfield
}

interface TouchTracker {
  id: number;
  startY: number;
  currentY: number;
  previousY: number;
  lastUpdateTime: number;
  velocity: number;
  isLeft: boolean;
  smoothedAxis: number;
}

export class InputManager {
  private currentInput: InputState = {
    leftPaddleAxis: INPUT_CONFIG.AXIS_RANGE.NEUTRAL,
    rightPaddleAxis: INPUT_CONFIG.AXIS_RANGE.NEUTRAL,
    leftButtons: 0,
    rightButtons: 0,
  };

  private keysPressed = new Set<string>();
  private activeTouches = new Map<number, TouchTracker>();
  private desktopMobileMode = false;
  private updateFABVisibilityCallback: (() => void) | null = null;
  private currentSensitivity: keyof typeof INPUT_CONFIG.TOUCH_SENSITIVITY =
    "DEFAULT";
  constructor() {
    this.loadMobileModeSetting();
    this.loadSensitivitySetting();
  }

  /**
   * Initialize all input systems
   */
  initialize(): void {
    this.setupKeyboardInput();
    this.setupTouchInput();
    this.setupFullscreenToggle();
    this.setupMobileModeToggle();
    this.setupMobileFABs();
  }

  /**
   * Get current input state
   */
  getInputState(): InputState {
    return { ...this.currentInput };
  }

  /**
   * Set callback for FAB visibility updates
   */
  setFABVisibilityCallback(callback: () => void): void {
    this.updateFABVisibilityCallback = callback;
  }

  /**
   * Check if desktop mobile mode is enabled
   */
  isDesktopMobileMode(): boolean {
    return this.desktopMobileMode;
  }

  /**
   * Setup keyboard input handling
   */
  private setupKeyboardInput(): void {
    const updateKeyboardInput = () => {
      // Left paddle (W/S keys only)
      if (this.keysPressed.has("KeyW")) {
        this.currentInput.leftPaddleAxis = INPUT_CONFIG.AXIS_RANGE.MAX;
      } else if (this.keysPressed.has("KeyS")) {
        this.currentInput.leftPaddleAxis = INPUT_CONFIG.AXIS_RANGE.MIN;
      } else {
        this.currentInput.leftPaddleAxis = INPUT_CONFIG.AXIS_RANGE.NEUTRAL;
      }

      // Right paddle (Arrow keys only)
      if (this.keysPressed.has("ArrowUp")) {
        this.currentInput.rightPaddleAxis = INPUT_CONFIG.AXIS_RANGE.MAX;
      } else if (this.keysPressed.has("ArrowDown")) {
        this.currentInput.rightPaddleAxis = INPUT_CONFIG.AXIS_RANGE.MIN;
      } else {
        this.currentInput.rightPaddleAxis = INPUT_CONFIG.AXIS_RANGE.NEUTRAL;
      }

      // Buttons (SPACE for ready, etc.)
      if (this.keysPressed.has("Space")) {
        this.currentInput.leftButtons |= INPUT_CONFIG.BUTTON_BITS.READY;
        this.currentInput.rightButtons |= INPUT_CONFIG.BUTTON_BITS.READY;
      } else {
        this.currentInput.leftButtons &= ~INPUT_CONFIG.BUTTON_BITS.READY;
        this.currentInput.rightButtons &= ~INPUT_CONFIG.BUTTON_BITS.READY;
      }
    };

    document.addEventListener("keydown", (event) => {
      this.keysPressed.add(event.code);
      updateKeyboardInput();
    });

    document.addEventListener("keyup", (event) => {
      this.keysPressed.delete(event.code);
      updateKeyboardInput();
    });
  }

  /**
   * Setup touch input handling for mobile controls
   */
  private setupTouchInput(): void {
    const leftZone = document.getElementById("left-touch-zone");
    const rightZone = document.getElementById("right-touch-zone");

    if (!leftZone || !rightZone) return;

    const handleTouchStart = (event: TouchEvent, isLeft: boolean): void => {
      event.preventDefault();
      const zone = isLeft ? leftZone : rightZone;
      const currentTime = performance.now();

      for (let i = 0; i < event.changedTouches.length; i++) {
        const touch = event.changedTouches[i];
        this.activeTouches.set(touch.identifier, {
          id: touch.identifier,
          startY: touch.clientY,
          currentY: touch.clientY,
          previousY: touch.clientY,
          lastUpdateTime: currentTime,
          velocity: 0,
          isLeft: isLeft,
          smoothedAxis: 0,
        });
      }

      zone.classList.add("active");
      this.updateTouchInput();
    };

    const handleTouchMove = (event: TouchEvent): void => {
      event.preventDefault();
      const currentTime = performance.now();

      for (let i = 0; i < event.changedTouches.length; i++) {
        const touch = event.changedTouches[i];
        const tracker = this.activeTouches.get(touch.identifier);

        if (tracker) {
          // Update position and calculate velocity
          tracker.previousY = tracker.currentY;
          tracker.currentY = touch.clientY;

          const deltaTime = currentTime - tracker.lastUpdateTime;
          if (deltaTime > 0) {
            tracker.velocity =
              (tracker.currentY - tracker.previousY) / deltaTime;
          }
          tracker.lastUpdateTime = currentTime;
        }
      }

      this.updateTouchInput();
    };

    const handleTouchEnd = (event: TouchEvent): void => {
      event.preventDefault();

      for (let i = 0; i < event.changedTouches.length; i++) {
        const touch = event.changedTouches[i];
        const tracker = this.activeTouches.get(touch.identifier);

        if (tracker) {
          this.activeTouches.delete(touch.identifier);

          // Check if this zone has no more active touches
          const hasLeftTouches = Array.from(this.activeTouches.values()).some(
            (t) => t.isLeft
          );
          const hasRightTouches = Array.from(this.activeTouches.values()).some(
            (t) => !t.isLeft
          );

          if (!hasLeftTouches) {
            leftZone.classList.remove("active");
            this.currentInput.leftPaddleAxis = INPUT_CONFIG.AXIS_RANGE.NEUTRAL;
          }
          if (!hasRightTouches) {
            rightZone.classList.remove("active");
            this.currentInput.rightPaddleAxis = INPUT_CONFIG.AXIS_RANGE.NEUTRAL;
          }
        }
      }
    };

    // Left zone event listeners
    leftZone.addEventListener("touchstart", (e) => handleTouchStart(e, true), {
      passive: false,
    });
    leftZone.addEventListener("touchmove", handleTouchMove, { passive: false });
    leftZone.addEventListener("touchend", handleTouchEnd, { passive: false });
    leftZone.addEventListener("touchcancel", handleTouchEnd, {
      passive: false,
    });

    // Right zone event listeners
    rightZone.addEventListener(
      "touchstart",
      (e) => handleTouchStart(e, false),
      {
        passive: false,
      }
    );
    rightZone.addEventListener("touchmove", handleTouchMove, {
      passive: false,
    });
    rightZone.addEventListener("touchend", handleTouchEnd, { passive: false });
    rightZone.addEventListener("touchcancel", handleTouchEnd, {
      passive: false,
    });
  }

  /**
   * Instant positioning - paddle moves exactly to thumb position
   */
  private updateTouchInput(): void {
    const leftZone = document.getElementById("left-touch-zone");
    const rightZone = document.getElementById("right-touch-zone");

    if (!leftZone || !rightZone) return;

    let leftAxisValue = INPUT_CONFIG.AXIS_RANGE.NEUTRAL;
    let rightAxisValue = INPUT_CONFIG.AXIS_RANGE.NEUTRAL;

    // Process active touches with instant 1:1 position mapping
    for (const tracker of this.activeTouches.values()) {
      const touchZone = tracker.isLeft ? leftZone : rightZone;
      const zoneRect = touchZone.getBoundingClientRect();

      // Calculate position within touch zone (0 = top, 1 = bottom)
      const relativeY = (tracker.currentY - zoneRect.top) / zoneRect.height;
      const clampedY = Math.max(0, Math.min(1, relativeY));

      // Direct 1:1 position mapping: thumb position = paddle position
      // 0 (top) = MAX (up), 1 (bottom) = MIN (down), 0.5 (center) = NEUTRAL
      const axisValue = Math.round(
        INPUT_CONFIG.AXIS_RANGE.MAX -
          clampedY * (INPUT_CONFIG.AXIS_RANGE.MAX - INPUT_CONFIG.AXIS_RANGE.MIN)
      );

      // Instant assignment - paddle moves exactly to thumb position
      if (tracker.isLeft) {
        leftAxisValue = axisValue;
      } else {
        rightAxisValue = axisValue;
      }
    }

    // Instant update - no processing delays
    this.currentInput.leftPaddleAxis = leftAxisValue;
    this.currentInput.rightPaddleAxis = rightAxisValue;
  }

  /**
   * Setup fullscreen toggle functionality
   */
  private setupFullscreenToggle(): void {
    const fullscreenBtn = document.getElementById("fullscreen-btn");
    if (!fullscreenBtn) return;

    const updateFullscreenButton = (): void => {
      const isFullscreen = !!(
        document.fullscreenElement ||
        (document as any).webkitFullscreenElement ||
        (document as any).mozFullScreenElement
      );

      fullscreenBtn.textContent = isFullscreen
        ? "⛉ Exit Fullscreen"
        : "⛶ Fullscreen";
    };

    const toggleFullscreen = async (): Promise<void> => {
      try {
        const isFullscreen = !!(
          document.fullscreenElement ||
          (document as any).webkitFullscreenElement ||
          (document as any).mozFullScreenElement
        );

        if (isFullscreen) {
          // Exit fullscreen
          if (document.exitFullscreen) {
            await document.exitFullscreen();
          } else if ((document as any).webkitExitFullscreen) {
            await (document as any).webkitExitFullscreen();
          } else if ((document as any).mozCancelFullScreen) {
            await (document as any).mozCancelFullScreen();
          }
        } else {
          // Enter fullscreen
          const element = document.documentElement;
          if (element.requestFullscreen) {
            await element.requestFullscreen();
          } else if ((element as any).webkitRequestFullscreen) {
            await (element as any).webkitRequestFullscreen();
          } else if ((element as any).mozRequestFullScreen) {
            await (element as any).mozRequestFullScreen();
          }
        }
      } catch (error) {
        console.error("Fullscreen toggle failed:", error);
      }
    };

    // Button click handler
    fullscreenBtn.addEventListener("click", toggleFullscreen);

    // Listen for fullscreen changes
    document.addEventListener("fullscreenchange", updateFullscreenButton);
    document.addEventListener("webkitfullscreenchange", updateFullscreenButton);
    document.addEventListener("mozfullscreenchange", updateFullscreenButton);

    // Initial state
    updateFullscreenButton();
  }

  /**
   * Setup mobile mode toggle functionality
   */
  private setupMobileModeToggle(): void {
    const toggleButton = document.getElementById("mobile-mode-toggle");
    if (!toggleButton) return;

    // Apply saved state
    if (this.desktopMobileMode) {
      toggleButton.classList.add("active");
      toggleButton.textContent = "📱 Desktop Mode";
      document.body.classList.add("desktop-mobile-mode");
    }

    // Toggle button event listener
    toggleButton.addEventListener("click", () => {
      this.desktopMobileMode = !this.desktopMobileMode;

      // Update button appearance and text
      if (this.desktopMobileMode) {
        toggleButton.classList.add("active");
        toggleButton.textContent = "📱 Desktop Mode";
        document.body.classList.add("desktop-mobile-mode");
      } else {
        toggleButton.classList.remove("active");
        toggleButton.textContent = "📱 Mobile Mode";
        document.body.classList.remove("desktop-mobile-mode");
      }

      // Save state
      localStorage.setItem(
        STORAGE_KEYS.DESKTOP_MOBILE_MODE,
        this.desktopMobileMode.toString()
      );

      // Update FAB visibility
      if (this.updateFABVisibilityCallback) {
        this.updateFABVisibilityCallback();
      }
    });
  }

  /**
   * Setup mobile floating action buttons
   */
  private setupMobileFABs(): void {
    const readyButton = document.getElementById(
      "fab-ready"
    ) as HTMLButtonElement;
    const menuButton = document.getElementById("fab-menu") as HTMLButtonElement;
    const quitButton = document.getElementById("fab-quit") as HTMLButtonElement;

    if (!readyButton || !menuButton || !quitButton) return;

    // Ready/Start button handler
    readyButton.addEventListener("click", () => {
      this.simulateSpacePress();
      this.addButtonFeedback(readyButton);
    });

    // Menu button handler
    menuButton.addEventListener("click", () => {
      this.addButtonFeedback(menuButton);
      console.log("Menu button pressed - ESC functionality would go here");
    });

    // Quit button handler
    quitButton.addEventListener("click", () => {
      this.addButtonFeedback(quitButton);
      console.log("Quit button pressed - Q key functionality would go here");
    });
  }

  /**
   * Simulate SPACE key press for ready/start
   */
  private simulateSpacePress(): void {
    // Set button state
    this.currentInput.leftButtons |= INPUT_CONFIG.BUTTON_BITS.READY;
    this.currentInput.rightButtons |= INPUT_CONFIG.BUTTON_BITS.READY;

    // Release button after short delay
    setTimeout(() => {
      this.currentInput.leftButtons &= ~INPUT_CONFIG.BUTTON_BITS.READY;
      this.currentInput.rightButtons &= ~INPUT_CONFIG.BUTTON_BITS.READY;
    }, 100);
  }

  /**
   * Add visual feedback to button press
   */
  private addButtonFeedback(button: HTMLElement): void {
    const originalTransform = button.style.transform;
    button.style.transform = `${originalTransform} scale(${MOBILE_CONFIG.BUTTON_SCALE_FEEDBACK})`;

    setTimeout(() => {
      button.style.transform = originalTransform;
    }, MOBILE_CONFIG.TOUCH_FEEDBACK_DURATION);
  }

  /**
   * Update FAB visibility based on game state and device
   */
  updateFABVisibility(gameStatus?: string): void {
    const readyButton = document.getElementById("fab-ready");
    const menuButton = document.getElementById("fab-menu");
    const quitButton = document.getElementById("fab-quit");

    if (!readyButton || !menuButton || !quitButton) return;

    const isMobileOrToggled = this.isMobileDevice() || this.desktopMobileMode;

    if (!isMobileOrToggled) {
      // Hide all FABs when not in mobile mode
      readyButton.classList.remove("visible");
      menuButton.classList.remove("visible");
      quitButton.classList.remove("visible");
      return;
    }

    // Show different buttons based on game state
    if (gameStatus === "Lobby") {
      readyButton.classList.add("visible");
      menuButton.classList.remove("visible");
      quitButton.classList.add("visible");
    } else {
      readyButton.classList.remove("visible");
      menuButton.classList.add("visible");
      quitButton.classList.add("visible");
    }
  }

  /**
   * Check if we're on a mobile device
   */
  private isMobileDevice(): boolean {
    return (
      /Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
        navigator.userAgent
      ) ||
      window.matchMedia(`(max-width: ${VIEWPORT_CONFIG.MOBILE_MAX_WIDTH}px)`)
        .matches ||
      window.matchMedia(
        `(orientation: landscape) and (max-height: ${VIEWPORT_CONFIG.MOBILE_LANDSCAPE_MAX_HEIGHT}px)`
      ).matches
    );
  }

  /**
   * Load mobile mode setting from localStorage
   */
  private loadMobileModeSetting(): void {
    const savedState = localStorage.getItem(STORAGE_KEYS.DESKTOP_MOBILE_MODE);
    this.desktopMobileMode = savedState === "true";
  }

  /**
   * Load sensitivity setting from localStorage
   */
  private loadSensitivitySetting(): void {
    const savedState = localStorage.getItem(
      STORAGE_KEYS.TOUCH_SENSITIVITY_LEVEL
    );
    if (savedState && savedState in INPUT_CONFIG.TOUCH_SENSITIVITY) {
      this.currentSensitivity =
        savedState as keyof typeof INPUT_CONFIG.TOUCH_SENSITIVITY;
    }
  }

  /**
   * Set touch sensitivity level
   */
  setSensitivity(level: keyof typeof INPUT_CONFIG.TOUCH_SENSITIVITY): void {
    this.currentSensitivity = level;
    localStorage.setItem(STORAGE_KEYS.TOUCH_SENSITIVITY_LEVEL, level);
    console.log(
      `Touch sensitivity set to: ${level} (${INPUT_CONFIG.TOUCH_SENSITIVITY[level]})`
    );
  }

  /**
   * Get current sensitivity level
   */
  getCurrentSensitivity(): keyof typeof INPUT_CONFIG.TOUCH_SENSITIVITY {
    return this.currentSensitivity;
  }
}
