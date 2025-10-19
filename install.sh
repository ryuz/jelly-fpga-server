#!/usr/bin/env bash

set -euo pipefail

# jelly-fpga-server installer
# - Builds the binary (if cargo is available)
# - Installs it to /usr/local/bin
# - Creates an environment file at /etc/default/jelly-fpga-server (OPTIONS)
# - Registers and starts a systemd service: jelly-fpga-server.service

APP_NAME="jelly-fpga-server"
BIN_INSTALL_PATH="/usr/local/bin/${APP_NAME}"
ENV_FILE="/etc/default/${APP_NAME}"
SERVICE_FILE="/etc/systemd/system/${APP_NAME}.service"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="${SCRIPT_DIR}"

echo "[1/5] Building ${APP_NAME} (release) if cargo is available..."
if command -v cargo >/dev/null 2>&1; then
  (cd "${PROJECT_ROOT}" && cargo build --release)
else
  echo "cargo not found. Skipping build step and expecting an existing binary at target/release/${APP_NAME}."
fi

BUILD_BIN="${PROJECT_ROOT}/target/release/${APP_NAME}"
if [[ ! -x "${BUILD_BIN}" ]]; then
  echo "Error: Built binary not found at ${BUILD_BIN}. Ensure cargo is installed and build succeeds, or place the binary there."
  exit 1
fi

echo "[2/5] Installing binary to ${BIN_INSTALL_PATH} (requires sudo)."
sudo install -Dm755 "${BUILD_BIN}" "${BIN_INSTALL_PATH}"

echo "[3/5] Writing default environment file to ${ENV_FILE} (requires sudo)."
if [[ ! -f "${ENV_FILE}" ]]; then
  sudo tee "${ENV_FILE}" >/dev/null <<'EOF'
# Environment for jelly-fpga-server
# Set OPTIONS to pass flags to the server. Examples:
#   OPTIONS="--port 8051 --verbose 0"
#   OPTIONS="--external --port 8051 --verbose 1"
#   OPTIONS="--external --allow-sudo --port 8051 --verbose 1"
# See: ./jelly-fpga-server --help

OPTIONS="--port 8051 --verbose 0"
EOF
else
  echo "Environment file already exists, keeping existing ${ENV_FILE}."
fi

echo "[4/5] Creating systemd unit at ${SERVICE_FILE} (requires sudo)."
sudo tee "${SERVICE_FILE}" >/dev/null <<EOF
[Unit]
Description=Jelly FPGA gRPC Server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=-${ENV_FILE}
ExecStart=${BIN_INSTALL_PATH} $OPTIONS
Restart=on-failure
RestartSec=2s
Nice=5

[Install]
WantedBy=multi-user.target
EOF

echo "[5/5] Enabling and starting service (requires sudo)."
sudo systemctl daemon-reload
sudo systemctl enable "${APP_NAME}.service"
sudo systemctl restart "${APP_NAME}.service"

echo "Done. Check status with: sudo systemctl status ${APP_NAME}.service"
