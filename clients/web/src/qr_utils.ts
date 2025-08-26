/**
 * QR code generation and scanning utilities for SDP exchange
 */

import QrScanner from "qr-scanner";

/**
 * Generate a QR code data URL for the given text using QR Server API
 */
export function generateQRCode(text: string, size: number = 200): string {
  try {
    // Use qr-server.com API for QR code generation
    // This is a reliable service for generating QR codes
    const encodedText = encodeURIComponent(text);
    return `https://api.qrserver.com/v1/create-qr-code/?size=${size}x${size}&data=${encodedText}`;
  } catch (error) {
    console.error("Failed to generate QR code:", error);
    return "";
  }
}

/**
 * Generate QR code as canvas element (fallback implementation)
 */
export function generateQRCodeCanvas(text: string): HTMLCanvasElement | null {
  try {
    // Simple QR code placeholder using canvas
    const canvas = document.createElement("canvas");
    canvas.width = 200;
    canvas.height = 200;
    const ctx = canvas.getContext("2d");

    if (!ctx) return null;

    // Draw a simple pattern as placeholder
    ctx.fillStyle = "#000000";
    ctx.fillRect(0, 0, 200, 200);
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(10, 10, 180, 180);

    // Add text indicating this is a QR code
    ctx.fillStyle = "#000000";
    ctx.font = "12px monospace";
    ctx.textAlign = "center";
    ctx.fillText("QR CODE", 100, 100);
    ctx.fillText("(Generated)", 100, 120);

    return canvas;
  } catch (error) {
    console.error("Failed to generate canvas QR code:", error);
    return null;
  }
}

/**
 * Create QR code element for SDP display
 */
export function createQRCodeElement(
  sdp: string,
  containerId: string
): HTMLElement {
  const container = document.getElementById(containerId);
  if (!container) {
    throw new Error(`Container ${containerId} not found`);
  }

  // Clear existing QR code
  const existingQr = container.querySelector(".qr-code-display");
  if (existingQr) {
    existingQr.remove();
  }

  // Create QR code wrapper
  const qrWrapper = document.createElement("div");
  qrWrapper.className = "qr-code-display";

  // Generate QR code using API
  const qrUrl = generateQRCode(sdp, 200);

  if (qrUrl) {
    // Create QR code image
    const qrImage = document.createElement("img");
    qrImage.src = qrUrl;
    qrImage.alt = "QR Code for SDP";
    qrImage.className = "qr-code-image";
    qrImage.loading = "lazy";

    // Add error handling for image load
    qrImage.onerror = () => {
      const errorDiv = document.createElement("div");
      errorDiv.className = "qr-error";
      errorDiv.textContent = "Failed to load QR code";
      qrWrapper.appendChild(errorDiv);
    };

    // Add scan instruction
    const instruction = document.createElement("div");
    instruction.className = "qr-instruction";
    instruction.textContent = "Scan this QR code with the other device";

    qrWrapper.appendChild(qrImage);
    qrWrapper.appendChild(instruction);
  } else {
    // Fallback if QR generation fails
    const error = document.createElement("div");
    error.className = "qr-error";
    error.textContent = "QR code generation failed";
    qrWrapper.appendChild(error);
  }

  container.appendChild(qrWrapper);
  return qrWrapper;
}

/**
 * Create toggle button for switching between QR and text display
 */
export function createDisplayToggle(
  textElement: HTMLElement,
  qrContainer: HTMLElement,
  onToggle: (showQR: boolean) => void
): HTMLButtonElement {
  const toggleBtn = document.createElement("button");
  toggleBtn.className = "qr-toggle-btn";
  toggleBtn.textContent = "📱 Show QR Code";

  let showingQR = false;

  toggleBtn.addEventListener("click", () => {
    showingQR = !showingQR;

    if (showingQR) {
      textElement.style.display = "none";
      qrContainer.style.display = "block";
      toggleBtn.textContent = "📄 Show Text";
    } else {
      textElement.style.display = "block";
      qrContainer.style.display = "none";
      toggleBtn.textContent = "📱 Show QR Code";
    }

    onToggle(showingQR);
  });

  return toggleBtn;
}

/**
 * Setup QR code display for SDP exchange
 */
export function setupQRDisplay(
  sdp: string,
  textAreaId: string,
  containerId: string,
  label: string
): void {
  const textArea = document.getElementById(textAreaId) as HTMLTextAreaElement;
  const container = document.getElementById(containerId);

  if (!textArea || !container) {
    console.error(`Elements not found: ${textAreaId}, ${containerId}`);
    return;
  }

  // Create QR container if it doesn't exist
  let qrContainer = container.querySelector(".qr-container") as HTMLElement;
  if (!qrContainer) {
    qrContainer = document.createElement("div");
    qrContainer.className = "qr-container";
    qrContainer.style.display = "none";
    container.appendChild(qrContainer);
  }

  // Generate QR code
  createQRCodeElement(sdp, qrContainer.id || containerId);

  // Create toggle button if it doesn't exist
  let toggleBtn = container.querySelector(
    ".qr-toggle-btn"
  ) as HTMLButtonElement;
  if (!toggleBtn) {
    toggleBtn = createDisplayToggle(textArea, qrContainer, (showQR) => {
      console.log(`Toggled to ${showQR ? "QR" : "text"} display for ${label}`);
    });

    // Insert toggle button after the label
    const labelElement = container.querySelector(".sdp-label");
    if (labelElement && labelElement.parentNode) {
      labelElement.parentNode.insertBefore(toggleBtn, labelElement.nextSibling);
    } else {
      container.insertBefore(toggleBtn, textArea);
    }
  }
}

/**
 * Update QR display with new SDP
 */
export function updateQRDisplay(sdp: string, containerId: string): void {
  if (!sdp) return;

  const container = document.getElementById(containerId);
  if (!container) return;

  const qrContainer = container.querySelector(".qr-container") as HTMLElement;
  if (qrContainer) {
    // Clear and regenerate QR code
    qrContainer.innerHTML = "";
    createQRCodeElement(sdp, qrContainer.id || containerId);
  }
}

// ============================================================================
// QR CODE SCANNING FUNCTIONALITY
// ============================================================================

let currentScanner: QrScanner | null = null;

/**
 * Detect if running on mobile device
 */
function isMobileDevice(): boolean {
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
    navigator.userAgent
  );
}

/**
 * Check if HTTPS is being used
 */
function isHTTPS(): boolean {
  return location.protocol === "https:" || location.hostname === "localhost";
}

/**
 * Get available cameras
 */
async function getAvailableCameras(): Promise<MediaDeviceInfo[]> {
  try {
    const devices = await navigator.mediaDevices.enumerateDevices();
    return devices.filter((device) => device.kind === "videoinput");
  } catch (error) {
    console.warn("Could not enumerate cameras:", error);
    return [];
  }
}

/**
 * Create a modal overlay for QR code scanning
 */
export function createScannerModal(): HTMLElement {
  const modal = document.createElement("div");
  modal.className = "qr-scanner-modal";
  modal.innerHTML = `
    <div class="qr-scanner-overlay">
      <div class="qr-scanner-container">
        <div class="qr-scanner-header">
          <h3>Scan QR Code</h3>
          <button class="qr-scanner-close" title="Close Scanner">✕</button>
        </div>
        <div class="qr-scanner-content">
          <video class="qr-scanner-video"></video>
          <div class="qr-scanner-target">
            <div class="qr-scanner-corners">
              <div class="corner top-left"></div>
              <div class="corner top-right"></div>
              <div class="corner bottom-left"></div>
              <div class="corner bottom-right"></div>
            </div>
          </div>
          <div class="qr-scanner-instructions">
            Position QR code within the frame
          </div>
        </div>
        <div class="qr-scanner-status">
          <div class="qr-scanner-message">Preparing camera...</div>
        </div>
        <div class="qr-scanner-actions">
          <button class="qr-scanner-btn qr-scanner-retry" style="display: none;">Retry</button>
          <button class="qr-scanner-btn qr-scanner-cancel">Cancel</button>
        </div>
      </div>
    </div>
  `;

  return modal;
}

/**
 * Show scanning status message
 */
function updateScannerStatus(
  message: string,
  isError: boolean = false,
  showRetry: boolean = false
): void {
  const statusElement = document.querySelector(".qr-scanner-message");
  const retryBtn = document.querySelector(
    ".qr-scanner-retry"
  ) as HTMLButtonElement;

  if (statusElement) {
    statusElement.textContent = message;
    statusElement.className = `qr-scanner-message ${isError ? "error" : ""}`;
  }

  if (retryBtn) {
    retryBtn.style.display = showRetry ? "inline-block" : "none";
  }
}

/**
 * Pre-flight checks before starting camera
 */
async function performPreflightChecks(): Promise<string | null> {
  // Check HTTPS requirement
  if (!isHTTPS()) {
    return "Camera access requires HTTPS. Please use https:// or access via localhost.";
  }

  // Check if cameras exist
  const cameras = await getAvailableCameras();
  if (cameras.length === 0) {
    return "No cameras found on this device.";
  }

  // Check basic camera support
  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    return "Camera access not supported in this browser.";
  }

  // Check QR Scanner library support
  try {
    const hasCamera = await QrScanner.hasCamera();
    if (!hasCamera) {
      return "No camera available for QR scanning.";
    }
  } catch (error) {
    return "QR scanning not supported in this browser.";
  }

  return null; // All checks passed
}

/**
 * Try starting camera with different constraints
 */
async function attemptCameraStart(
  scanner: QrScanner,
  attempts: Array<{ camera?: string; description: string }>
): Promise<void> {
  let lastError: Error | null = null;

  for (const attempt of attempts) {
    try {
      updateScannerStatus(`Trying ${attempt.description}...`);

      if (attempt.camera) {
        scanner.setCamera(attempt.camera as any);
      }

      await scanner.start();
      updateScannerStatus("Point camera at QR code");
      return; // Success!
    } catch (error) {
      console.warn(`Failed to start ${attempt.description}:`, error);
      lastError = error as Error;

      // Stop scanner before trying next attempt
      try {
        await scanner.stop();
      } catch (stopError) {
        // Ignore stop errors
      }
    }
  }

  // All attempts failed
  throw lastError || new Error("Failed to start camera");
}

/**
 * Start QR code scanning with enhanced mobile support
 */
export async function startQRScanning(
  onScanSuccess: (result: string) => void,
  onScanError: (error: Error) => void
): Promise<void> {
  const modal = createScannerModal();
  document.body.appendChild(modal);

  const video = modal.querySelector(".qr-scanner-video") as HTMLVideoElement;
  const closeBtn = modal.querySelector(
    ".qr-scanner-close"
  ) as HTMLButtonElement;
  const cancelBtn = modal.querySelector(
    ".qr-scanner-cancel"
  ) as HTMLButtonElement;
  const retryBtn = modal.querySelector(
    ".qr-scanner-retry"
  ) as HTMLButtonElement;

  const cleanup = () => {
    if (currentScanner) {
      try {
        currentScanner.stop();
        currentScanner.destroy();
      } catch (error) {
        console.warn("Error during scanner cleanup:", error);
      }
      currentScanner = null;
    }
    modal.remove();
  };

  const handleClose = () => {
    cleanup();
    onScanError(new Error("Scan cancelled by user"));
  };

  const startScanning = async () => {
    try {
      updateScannerStatus("Checking camera availability...");

      // Perform pre-flight checks
      const preflightError = await performPreflightChecks();
      if (preflightError) {
        throw new Error(preflightError);
      }

      updateScannerStatus("Initializing scanner...");

      // Create QR Scanner instance
      currentScanner = new QrScanner(
        video,
        (result) => {
          cleanup();
          document.removeEventListener("keydown", handleKeyDown);
          // Handle both string and object result formats
          const resultData = typeof result === "string" ? result : result.data;
          onScanSuccess(resultData);
        },
        {
          highlightScanRegion: true,
          highlightCodeOutline: true,
          maxScansPerSecond: 5,
        }
      );

      // Prepare camera attempts with fallback options
      const cameraAttempts = [];

      if (isMobileDevice()) {
        // Mobile: try rear camera first, then front camera
        cameraAttempts.push(
          { camera: "environment", description: "rear camera" },
          { camera: "user", description: "front camera" },
          { description: "default camera" }
        );
      } else {
        // Desktop: try default first, then specific cameras
        cameraAttempts.push(
          { description: "default camera" },
          { camera: "user", description: "user camera" },
          { camera: "environment", description: "environment camera" }
        );
      }

      // Attempt to start camera with fallbacks
      await attemptCameraStart(currentScanner, cameraAttempts);
    } catch (error) {
      console.error("QR Scanner initialization failed:", error);

      let errorMessage = "Failed to start camera";
      let showRetry = false;

      if (error instanceof Error) {
        switch (error.name) {
          case "NotAllowedError":
            errorMessage = isMobileDevice()
              ? "Camera permission denied. Please allow camera access in your browser settings and try again."
              : "Camera permission denied. Please allow camera access and try again.";
            showRetry = true;
            break;

          case "NotFoundError":
            errorMessage = "No camera found on this device.";
            break;

          case "NotSupportedError":
            errorMessage = "QR scanning not supported in this browser.";
            break;

          case "NotReadableError":
            errorMessage = "Camera is already in use by another application.";
            showRetry = true;
            break;

          case "OverconstrainedError":
            errorMessage =
              "Camera constraints could not be satisfied. Trying different settings...";
            showRetry = true;
            break;

          case "AbortError":
            errorMessage = "Camera access was interrupted.";
            showRetry = true;
            break;

          default:
            errorMessage = error.message || "Unknown camera error";
            showRetry = true;
        }
      }

      updateScannerStatus(errorMessage, true, showRetry);

      if (!showRetry) {
        // For permanent errors, close after a delay
        setTimeout(() => {
          cleanup();
          document.removeEventListener("keydown", handleKeyDown);
          onScanError(new Error(errorMessage));
        }, 3000);
      }
    }
  };

  // Event listeners
  closeBtn.addEventListener("click", handleClose);
  cancelBtn.addEventListener("click", handleClose);
  retryBtn.addEventListener("click", startScanning);

  // Handle clicking outside modal
  modal.addEventListener("click", (e) => {
    if (e.target === modal) {
      handleClose();
    }
  });

  // Handle escape key
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      handleClose();
      document.removeEventListener("keydown", handleKeyDown);
    }
  };
  document.addEventListener("keydown", handleKeyDown);

  // Start the scanning process
  await startScanning();
}

/**
 * Create a scan button for SDP input
 */
export function createScanButton(
  targetTextArea: HTMLTextAreaElement,
  label: string = "Scan QR Code"
): HTMLButtonElement {
  const scanBtn = document.createElement("button");
  scanBtn.className = "qr-scan-btn";
  scanBtn.innerHTML = `📷 ${label}`;
  scanBtn.type = "button";
  scanBtn.title = "Scan QR code to automatically fill this field";

  scanBtn.addEventListener("click", () => {
    scanBtn.disabled = true;
    scanBtn.textContent = "Starting camera...";

    startQRScanning(
      (scannedData) => {
        // Successfully scanned QR code
        targetTextArea.value = scannedData;
        targetTextArea.dispatchEvent(new Event("input", { bubbles: true }));

        scanBtn.disabled = false;
        scanBtn.innerHTML = `📷 ${label}`;

        // Visual feedback
        targetTextArea.classList.add("qr-scanned");
        setTimeout(() => {
          targetTextArea.classList.remove("qr-scanned");
        }, 1000);
      },
      (error) => {
        // Scan failed or cancelled
        scanBtn.disabled = false;
        scanBtn.innerHTML = `📷 ${label}`;

        if (error.message !== "Scan cancelled by user") {
          alert(`QR Scan Error: ${error.message}`);
        }
      }
    );
  });

  return scanBtn;
}

/**
 * Add scan button to an existing SDP input section
 */
export function addScanButtonToSection(
  textAreaId: string,
  containerId: string,
  label: string = "Scan QR Code"
): HTMLButtonElement | null {
  const textArea = document.getElementById(textAreaId) as HTMLTextAreaElement;
  const container = document.getElementById(containerId);

  if (!textArea || !container) {
    console.error(`Elements not found: ${textAreaId}, ${containerId}`);
    return null;
  }

  // Check if scan button already exists
  const existingScanBtn = container.querySelector(".qr-scan-btn");
  if (existingScanBtn) {
    return existingScanBtn as HTMLButtonElement;
  }

  const scanBtn = createScanButton(textArea, label);

  // Insert scan button after the text area
  if (textArea.nextSibling) {
    container.insertBefore(scanBtn, textArea.nextSibling);
  } else {
    container.appendChild(scanBtn);
  }

  return scanBtn;
}

/**
 * Setup complete QR functionality (both generation and scanning) for SDP exchange
 */
export function setupCompleteQRFunctionality(
  sdp: string,
  textAreaId: string,
  containerId: string,
  label: string,
  enableScanning: boolean = true
): void {
  // Setup existing QR display functionality
  setupQRDisplay(sdp, textAreaId, containerId, label);

  // Add scanning functionality if enabled and this is an input field
  if (enableScanning) {
    const textArea = document.getElementById(textAreaId) as HTMLTextAreaElement;
    if (textArea && !textArea.readOnly) {
      addScanButtonToSection(textAreaId, containerId, "Scan QR Code");
    }
  }
}
