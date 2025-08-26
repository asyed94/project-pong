// Constants and configuration for Pong Web Client

export const GAME_CONFIG = {
  TICK_RATE: 60,
  FRAME_RATE: 16.67, // 60 FPS
  FIXED_POINT_SCALE: 65536,
  DEFAULT_GAME_CONFIG: {
    paddle_half_h: 8192,
    paddle_speed: 196608,
    ball_speed: 32768,
    ball_speed_up: 68813,
    wall_thickness: 0,
    paddle_x: 3276,
    max_score: 11,
    seed: 0xc0ffee,
    tick_hz: 60,
    ball_radius: 2048,
    paddle_width: 1638,
  },
} as const;

// Simple, fast input configuration
export const FAST_INPUT_CONFIG = {
  DEAD_ZONE_PERCENTAGE: 0.01, // Minimal dead zone to prevent jitter
  TOUCH_SENSITIVITY: 1.2, // Slightly amplified sensitivity
} as const;

export const INPUT_CONFIG = {
  AXIS_RANGE: {
    MIN: -127 as number,
    MAX: 127 as number,
    NEUTRAL: 0 as number,
  },
  BUTTON_BITS: {
    READY: 1,
  },
  // Simplified sensitivity settings
  TOUCH_SENSITIVITY: {
    LOW: 2.5,
    MEDIUM: 4.0,
    HIGH: 6.0,
    DEFAULT: 6.0, // Higher default for faster response
  },
  // Input smoothing factor for AI
  INPUT_SMOOTHING: 0.15,
};

export const VIEWPORT_CONFIG = {
  MOBILE_LANDSCAPE_MAX_HEIGHT: 500,
  MOBILE_MAX_WIDTH: 768,
  VIEWPORT_UPDATE_DEBOUNCE: 50,
  ORIENTATION_CHANGE_DELAYS: [0, 100, 300, 600],
} as const;

export const FIELD_DIMENSIONS = {
  DESKTOP: { width: 68, height: 24 },
  MOBILE_PORTRAIT: { width: 42, height: 20 },
  MOBILE_LANDSCAPE: { width: 50, height: 16 },
} as const;

export const UNICODE_CHARS = {
  BALL: "●",
  PADDLE: "█",
  CENTER_LINE: "┊",
  BORDER: {
    TOP_LEFT: "╭",
    TOP_RIGHT: "╮",
    BOTTOM_LEFT: "╰",
    BOTTOM_RIGHT: "╯",
    HORIZONTAL: "─",
    VERTICAL: "│",
  },
} as const;

export const MOBILE_CONFIG = {
  DOUBLE_TAP_THRESHOLD: 300,
  FAB_UPDATE_INTERVAL: 1000,
  TOUCH_FEEDBACK_DURATION: 150,
  BUTTON_SCALE_FEEDBACK: 0.9,
} as const;

export const STORAGE_KEYS = {
  DESKTOP_MOBILE_MODE: "desktopMobileMode",
  TOUCH_SENSITIVITY_LEVEL: "touchSensitivityLevel",
  TOUCH_RESPONSIVE_MODE: "touchResponsiveMode",
} as const;
