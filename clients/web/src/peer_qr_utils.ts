/**
 * QR code generation and scanning utilities for Peer ID sharing
 * Optimized for short peer IDs instead of long SDP strings
 */

import QrScanner from "qr-scanner";
import { PeerIdUtils } from "./peer_transport.js";

/**
 * Generate QR code URL for peer ID using QR Server API
 */
export function generatePeerIdQRCode(
  peerId: string,
  size: number = 200
): string {
  try {
    const encodedPeerId = encodeURIComponent(peerId);
    return `https://api.qrserver.com/v1/create-qr-code/?size=${size}x${size}&data=${encodedPeerId}&ecc=M&margin=2`;
  } catch (error) {
    console.error("Failed to generate peer ID QR code:", error);
    return "";
  }
}

/**
 * Create QR code display element for peer ID
 */
export function createPeerIdQRDisplay(peerId: string): HTMLElement {
  const qrWrapper = document.createElement("div");
  qrWrapper.className = "peer-qr-display";

  // Generate QR code
  const qrUrl = generatePeerIdQRCode(peerId, 180);

  if (qrUrl) {
    const qrImage = document.createElement("img");
    qrImage.src = qrUrl;
    qrImage.alt = "QR Code for Game ID";
    qrImage.className = "peer-qr-image";
    qrImage.loading = "lazy";

    // Add error handling
    qrImage.onerror = () => {
      const errorDiv = document.createElement("div");
      errorDiv.className = "peer-qr-error";
      errorDiv.textContent = "Failed to load QR code";
      qrWrapper.appendChild(errorDiv);
    };

    // Add instruction
    const instruction = document.createElement("div");
    instruction.className = "peer-qr-instruction";
    instruction.innerHTML = `
      <strong>📱 Scan with your phone</strong><br>
      or share this Game ID: <code>${PeerIdUtils.formatPeerId(peerId)}</code>
    `;

    qrWrapper.appendChild(qrImage);
    qrWrapper.appendChild(instruction);
  } else {
    const error = document.createElement("div");
    error.className = "peer-qr-error";
    error.textContent = "QR code generation failed";
    qrWrapper.appendChild(error);
  }

  return qrWrapper;
}

/**
 * Create toggle button for QR code display
 */
export function createQRToggleButton(
  onToggle: (showQR: boolean) => void
): HTMLButtonElement {
  const toggleBtn = document.createElement("button");
  toggleBtn.className = "peer-qr-toggle";
  toggleBtn.innerHTML = "📱 Show QR Code";
  toggleBtn.type = "button";

  let showingQR = false;

  toggleBtn.addEventListener("click", () => {
    showingQR = !showingQR;
    toggleBtn.innerHTML = showingQR ? "📄 Show Text" : "📱 Show QR Code";
    onToggle(showingQR);
  });

  return toggleBtn;
}

/**
 * Setup QR code display for host screen
 */
export function setupHostQRDisplay(peerId: string, containerId: string): void {
  const container = document.getElementById(containerId);
  if (!container) {
    console.error(`Container ${containerId} not found`);
    return;
  }

  // Remove existing QR elements
  const existingQR = container.querySelector(".peer-qr-container");
  if (existingQR) {
    existingQR.remove();
  }

  // Create QR container
  const qrContainer = document.createElement("div");
  qrContainer.className = "peer-qr-container";
  qrContainer.style.display = "none"; // Hidden by default

  // Add QR code
  const qrDisplay = createPeerIdQRDisplay(peerId);
  qrContainer.appendChild(qrDisplay);

  // Find peer ID input and add QR container after it
  const peerIdInput = container.querySelector("#host-peer-id-output");
  if (peerIdInput && peerIdInput.parentNode) {
    peerIdInput.parentNode.insertBefore(qrContainer, peerIdInput.nextSibling);
  } else {
    container.appendChild(qrContainer);
  }

  // Add toggle button
  const toggleBtn = createQRToggleButton((showQR) => {
    qrContainer.style.display = showQR ? "block" : "none";

    // Add animation class
    if (showQR) {
      qrContainer.classList.add("fade-in");
      setTimeout(() => qrContainer.classList.remove("fade-in"), 300);
    }
  });

  // Insert toggle button after copy button or peer ID input
  const copyBtn = container.querySelector("#copy-peer-id-btn");
  if (copyBtn && copyBtn.parentNode) {
    copyBtn.parentNode.insertBefore(toggleBtn, copyBtn.nextSibling);
  } else if (peerIdInput && peerIdInput.parentNode) {
    peerIdInput.parentNode.insertBefore(toggleBtn, peerIdInput.nextSibling);
  }
}

// ============================================================================
// QR CODE SCANNING FOR GUEST SCREEN
// ============================================================================

let currentScanner: QrScanner | null = null;

/**
 * Check if device supports camera scanning
 */
export async function checkScanningSupport(): Promise<{
  supported: boolean;
  reason?: string;
}> {
  // Check HTTPS
  if (location.protocol !== "https:" && location.hostname !== "localhost") {
    return { supported: false, reason: "Camera requires HTTPS or localhost" };
  }

  // Check basic camera support
  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    return { supported: false, reason: "Camera not supported in this browser" };
  }

  try {
    const hasCamera = await QrScanner.hasCamera();
    if (!hasCamera) {
      return { supported: false, reason: "No camera found on this device" };
    }
  } catch (error) {
    return { supported: false, reason: "Camera access unavailable" };
  }

  return { supported: true };
}

/**
 * Create enhanced scanner modal for peer ID scanning
 */
function createPeerScannerModal(): HTMLElement {
  const modal = document.createElement("div");
  modal.className = "peer-scanner-modal";
  modal.innerHTML = `
    <div class="peer-scanner-overlay">
      <div class="peer-scanner-container">
        <div class="peer-scanner-header">
          <h3>📷 Scan Game ID</h3>
          <button class="peer-scanner-close" title="Close">✕</button>
        </div>
        <div class="peer-scanner-content">
          <video class="peer-scanner-video" playsinline></video>
          <div class="peer-scanner-viewfinder">
            <div class="viewfinder-corners">
              <div class="corner tl"></div>
              <div class="corner tr"></div>
              <div class="corner bl"></div>
              <div class="corner br"></div>
            </div>
            <div class="viewfinder-line"></div>
          </div>
        </div>
        <div class="peer-scanner-status">
          <div class="status-message">Starting camera...</div>
        </div>
        <div class="peer-scanner-actions">
          <button class="action-btn retry-btn" style="display: none;">🔄 Retry</button>
          <button class="action-btn cancel-btn">❌ Cancel</button>
        </div>
      </div>
    </div>
  `;
  return modal;
}

/**
 * Update scanner status message
 */
function updateScannerStatus(
  message: string,
  isError: boolean = false,
  showRetry: boolean = false
): void {
  const statusEl = document.querySelector(".status-message");
  const retryBtn = document.querySelector(".retry-btn") as HTMLButtonElement;

  if (statusEl) {
    statusEl.textContent = message;
    statusEl.className = `status-message ${isError ? "error" : ""}`;
  }

  if (retryBtn) {
    retryBtn.style.display = showRetry ? "inline-block" : "none";
  }
}

/**
 * Start camera with mobile optimization
 */
async function startCameraOptimized(scanner: QrScanner): Promise<void> {
  const isMobile = /Mobi|Android/i.test(navigator.userAgent);

  const attempts = isMobile
    ? [
        { camera: "environment", description: "rear camera" },
        { camera: "user", description: "front camera" },
        { description: "default camera" },
      ]
    : [
        { description: "default camera" },
        { camera: "user", description: "front camera" },
      ];

  let lastError: Error | null = null;

  for (const attempt of attempts) {
    try {
      updateScannerStatus(`Starting ${attempt.description}...`);

      if (attempt.camera) {
        scanner.setCamera(attempt.camera as any);
      }

      await scanner.start();
      updateScannerStatus("Point camera at QR code 📱");
      return;
    } catch (error) {
      console.warn(`Failed ${attempt.description}:`, error);
      lastError = error as Error;

      try {
        await scanner.stop();
      } catch (stopError) {
        // Ignore
      }
    }
  }

  throw lastError || new Error("Failed to start camera");
}

/**
 * Validate scanned data as peer ID
 */
function validateScannedPeerId(data: string): string | null {
  const cleaned = PeerIdUtils.cleanPeerId(data);

  if (PeerIdUtils.isValidPeerId(cleaned)) {
    return cleaned;
  }

  // Try to extract peer ID from URLs or other formats
  const peerIdMatch = data.match(/([a-zA-Z0-9_-]{8,20})/);
  if (peerIdMatch) {
    const extracted = PeerIdUtils.cleanPeerId(peerIdMatch[1]);
    if (PeerIdUtils.isValidPeerId(extracted)) {
      return extracted;
    }
  }

  return null;
}

/**
 * Start QR scanning for peer ID
 */
export async function startPeerIdScanning(
  onSuccess: (peerId: string) => void,
  onError: (error: string) => void
): Promise<void> {
  const modal = createPeerScannerModal();
  document.body.appendChild(modal);

  const video = modal.querySelector(".peer-scanner-video") as HTMLVideoElement;
  const closeBtn = modal.querySelector(
    ".peer-scanner-close"
  ) as HTMLButtonElement;
  const cancelBtn = modal.querySelector(".cancel-btn") as HTMLButtonElement;
  const retryBtn = modal.querySelector(".retry-btn") as HTMLButtonElement;

  const cleanup = () => {
    if (currentScanner) {
      try {
        currentScanner.stop();
        currentScanner.destroy();
      } catch (error) {
        console.warn("Scanner cleanup error:", error);
      }
      currentScanner = null;
    }
    modal.remove();
    document.removeEventListener("keydown", handleKeyDown);
  };

  const handleClose = () => {
    cleanup();
    onError("Scan cancelled");
  };

  const startScanning = async () => {
    try {
      // Check support first
      const support = await checkScanningSupport();
      if (!support.supported) {
        throw new Error(support.reason || "Scanning not supported");
      }

      updateScannerStatus("Initializing scanner...");

      currentScanner = new QrScanner(video, (result: any) => {
        const data =
          typeof result === "string" ? result : result.data || result;
        const peerId = validateScannedPeerId(data);

        if (peerId) {
          cleanup();
          onSuccess(peerId);
        } else {
          updateScannerStatus("Invalid Game ID - try again", false, false);
          // Continue scanning for valid peer ID
        }
      });

      // Set scanner options
      currentScanner.setInversionMode("both");

      await startCameraOptimized(currentScanner);
    } catch (error) {
      console.error("Scanner error:", error);

      let errorMessage = "Camera error";
      let showRetry = false;

      if (error instanceof Error) {
        switch (error.name) {
          case "NotAllowedError":
            errorMessage =
              "Camera permission denied. Please allow camera access and try again.";
            showRetry = true;
            break;
          case "NotFoundError":
            errorMessage = "No camera found on this device.";
            break;
          case "NotReadableError":
            errorMessage =
              "Camera is busy. Please close other apps using the camera.";
            showRetry = true;
            break;
          default:
            errorMessage = error.message || "Unknown error";
            showRetry = true;
        }
      }

      updateScannerStatus(errorMessage, true, showRetry);

      if (!showRetry) {
        setTimeout(() => {
          cleanup();
          onError(errorMessage);
        }, 3000);
      }
    }
  };

  // Event listeners
  closeBtn.addEventListener("click", handleClose);
  cancelBtn.addEventListener("click", handleClose);
  retryBtn.addEventListener("click", startScanning);

  modal.addEventListener("click", (e) => {
    if (e.target === modal) handleClose();
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") handleClose();
  };
  document.addEventListener("keydown", handleKeyDown);

  await startScanning();
}

/**
 * Create QR scan button for guest screen
 */
export function createPeerScanButton(
  targetInput: HTMLInputElement,
  onScanSuccess?: (peerId: string) => void
): HTMLButtonElement {
  const scanBtn = document.createElement("button");
  scanBtn.className = "peer-scan-btn";
  scanBtn.innerHTML = "📷 Scan QR";
  scanBtn.type = "button";
  scanBtn.title = "Scan QR code to get Game ID";

  scanBtn.addEventListener("click", async () => {
    // Check support first
    const support = await checkScanningSupport();
    if (!support.supported) {
      alert(`QR scanning not available: ${support.reason}`);
      return;
    }

    scanBtn.disabled = true;
    scanBtn.innerHTML = "📷 Starting...";

    startPeerIdScanning(
      (peerId) => {
        targetInput.value = PeerIdUtils.formatPeerId(peerId);
        targetInput.dispatchEvent(new Event("input", { bubbles: true }));

        // Visual feedback
        targetInput.classList.add("scan-success");
        setTimeout(() => targetInput.classList.remove("scan-success"), 1000);

        scanBtn.disabled = false;
        scanBtn.innerHTML = "📷 Scan QR";

        if (onScanSuccess) {
          onScanSuccess(peerId);
        }
      },
      (error) => {
        scanBtn.disabled = false;
        scanBtn.innerHTML = "📷 Scan QR";

        if (error !== "Scan cancelled") {
          alert(`Scan failed: ${error}`);
        }
      }
    );
  });

  return scanBtn;
}

/**
 * Setup QR scanning for guest screen
 */
export function setupGuestQRScanning(
  inputId: string,
  containerId: string
): void {
  const input = document.getElementById(inputId) as HTMLInputElement;
  const container = document.getElementById(containerId);

  if (!input || !container) {
    console.error(`Elements not found: ${inputId}, ${containerId}`);
    return;
  }

  // Check if scan button already exists
  if (container.querySelector(".peer-scan-btn")) {
    return;
  }

  const scanBtn = createPeerScanButton(input);

  // Insert scan button after input
  if (input.nextSibling) {
    container.insertBefore(scanBtn, input.nextSibling);
  } else {
    container.appendChild(scanBtn);
  }
}
