// Mobile controller module - handles mobile-specific functionality and optimizations

import { VIEWPORT_CONFIG, MOBILE_CONFIG } from "./constants.js";

export class MobileController {
  private updateTimeout: number | null = null;

  /**
   * Initialize all mobile systems
   */
  initialize(): void {
    this.initializeViewportHandler();
    this.initializeMobileOptimizations();
    console.log("Mobile controller initialized");
  }

  /**
   * Handle dynamic viewport height changes on mobile browsers
   * This addresses the issue where browser UI elements (address bar, navigation)
   * appear and disappear, changing the viewport height
   */
  private initializeViewportHandler(): void {
    // Function to set the actual viewport height
    const setViewportHeight = (): void => {
      // Clear any pending updates to debounce rapid changes
      if (this.updateTimeout) {
        clearTimeout(this.updateTimeout);
      }

      this.updateTimeout = window.setTimeout(() => {
        // Get the actual viewport height
        const vh = window.innerHeight * 0.01;

        // Set CSS custom property for actual viewport height
        document.documentElement.style.setProperty("--vh", `${vh}px`);

        console.log(
          `Viewport height updated: ${window.innerHeight}px (--vh: ${vh}px)`
        );

        // Force a reflow to ensure CSS updates are applied immediately
        document.documentElement.offsetHeight;

        this.updateTimeout = null;
      }, VIEWPORT_CONFIG.VIEWPORT_UPDATE_DEBOUNCE);
    };

    // Function for immediate viewport height updates (no debounce)
    const setViewportHeightImmediate = (): void => {
      const vh = window.innerHeight * 0.01;
      document.documentElement.style.setProperty("--vh", `${vh}px`);

      // Force reflow immediately
      document.documentElement.offsetHeight;

      console.log(
        `Viewport height updated immediately: ${window.innerHeight}px (--vh: ${vh}px)`
      );
    };

    // Set initial viewport height immediately
    setViewportHeightImmediate();

    // Listen for viewport changes with debouncing
    window.addEventListener("resize", setViewportHeight);

    // Handle orientation changes with progressive timing
    window.addEventListener("orientationchange", () => {
      // Multiple timed updates to catch various stages of orientation change
      VIEWPORT_CONFIG.ORIENTATION_CHANGE_DELAYS.forEach((delay) => {
        setTimeout(setViewportHeightImmediate, delay);
      });
    });

    // Visual Viewport API support for newer browsers
    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", setViewportHeight);
      window.visualViewport.addEventListener("scroll", setViewportHeight);
    }

    // Additional fallback for iOS Safari which sometimes needs extra nudging
    if (this.isIOSDevice()) {
      // iOS-specific handling
      window.addEventListener("focusin", setViewportHeight);
      window.addEventListener("focusout", () => {
        setTimeout(setViewportHeightImmediate, 300);
      });
    }

    console.log("Enhanced viewport handler initialized");
  }

  /**
   * Apply mobile-specific optimizations
   */
  private initializeMobileOptimizations(): void {
    if (this.isMobileDevice()) {
      console.log("Mobile device detected, applying optimizations");

      // Prevent zoom on double tap for better game experience
      this.setupDoubleTapPrevention();

      // Prevent pull-to-refresh on mobile
      document.body.style.overscrollBehavior = "none";

      // Disable text selection on mobile for better touch experience
      document.body.style.userSelect = "none";
      document.body.style.webkitUserSelect = "none";

      // Add mobile class for additional styling hooks
      document.body.classList.add("mobile-device");
    }
  }

  /**
   * Setup double tap prevention for better game experience
   */
  private setupDoubleTapPrevention(): void {
    let lastTouchEnd = 0;
    document.addEventListener(
      "touchend",
      (event) => {
        const now = new Date().getTime();
        if (now - lastTouchEnd <= MOBILE_CONFIG.DOUBLE_TAP_THRESHOLD) {
          event.preventDefault();
        }
        lastTouchEnd = now;
      },
      false
    );
  }

  /**
   * Check if we're on a mobile device
   */
  isMobileDevice(): boolean {
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
   * Check if we're on an iOS device
   */
  isIOSDevice(): boolean {
    return /iPhone|iPad|iPod/i.test(navigator.userAgent);
  }

  /**
   * Check if device is in landscape mobile mode
   */
  isLandscapeMobile(): boolean {
    return window.matchMedia(
      `(orientation: landscape) and (max-height: ${VIEWPORT_CONFIG.MOBILE_LANDSCAPE_MAX_HEIGHT}px)`
    ).matches;
  }

  /**
   * Check if device is in portrait mobile mode
   */
  isPortraitMobile(): boolean {
    return window.matchMedia(
      `(max-width: ${VIEWPORT_CONFIG.MOBILE_MAX_WIDTH}px) and (orientation: portrait)`
    ).matches;
  }

  /**
   * Get appropriate field size based on device orientation and size
   */
  getOptimalFieldSize(): { width: number; height: number } {
    if (this.isLandscapeMobile()) {
      return { width: 50, height: 16 }; // Compact dimensions for landscape mobile
    } else if (this.isPortraitMobile()) {
      return { width: 42, height: 20 }; // Medium dimensions for portrait mobile
    }
    return { width: 68, height: 24 }; // Desktop dimensions
  }

  /**
   * Get device-appropriate status text length
   */
  shouldUseCompactText(): boolean {
    return this.isLandscapeMobile();
  }

  /**
   * Apply device-specific CSS classes
   */
  updateDeviceClasses(): void {
    const body = document.body;

    // Remove existing mobile classes
    body.classList.remove("mobile-landscape", "mobile-portrait", "desktop");

    // Add appropriate class
    if (this.isLandscapeMobile()) {
      body.classList.add("mobile-landscape");
    } else if (this.isPortraitMobile()) {
      body.classList.add("mobile-portrait");
    } else {
      body.classList.add("desktop");
    }
  }

  /**
   * Setup orientation change listeners for dynamic updates
   */
  setupOrientationListeners(callback: () => void): void {
    window.addEventListener("resize", () => {
      this.updateDeviceClasses();
      callback();
    });

    window.addEventListener("orientationchange", () => {
      setTimeout(() => {
        this.updateDeviceClasses();
        callback();
      }, 100); // Delay to account for orientation change
    });

    // Initial update
    this.updateDeviceClasses();
  }

  /**
   * Force viewport recalculation (useful for troubleshooting)
   */
  forceViewportUpdate(): void {
    const vh = window.innerHeight * 0.01;
    document.documentElement.style.setProperty("--vh", `${vh}px`);
    document.documentElement.offsetHeight; // Force reflow
    console.log("Forced viewport update");
  }

  /**
   * Get current viewport info for debugging
   */
  getViewportInfo(): {
    innerHeight: number;
    innerWidth: number;
    vh: string | null;
    isMobile: boolean;
    isLandscape: boolean;
    isPortrait: boolean;
  } {
    return {
      innerHeight: window.innerHeight,
      innerWidth: window.innerWidth,
      vh: document.documentElement.style.getPropertyValue("--vh"),
      isMobile: this.isMobileDevice(),
      isLandscape: this.isLandscapeMobile(),
      isPortrait: this.isPortraitMobile(),
    };
  }
}
