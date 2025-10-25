#!/usr/bin/env bash

set -euo pipefail

# jelly-fpga-server binary installer
#
# Usage:
#   curl -sL https://raw.githubusercontent.com/ryuz/jelly-fpga-server/master/jelly-fpga-server/binst.sh | sudo bash

VERSION="v0.1.0"
REPO_URL="https://github.com/ryuz/jelly-fpga-server"
APP_NAME="jelly-fpga-server"

APP_NAME="jelly-fpga-server"
BIN_INSTALL_PATH="/usr/local/bin/${APP_NAME}"
ENV_FILE="/etc/default/${APP_NAME}"
SERVICE_FILE="/etc/systemd/system/${APP_NAME}.service"


# get target triple
ARCH=$(uname -m)
case $ARCH in
    "aarch64"|"arm64")
        # Kria K26
        TARGET_TRIPLE="aarch64-unknown-linux-gnu"
        ;;
    "arm"|"armv7l")
        # ZYBO
        TARGET_TRIPLE="arm-unknown-linux-gnueabihf"
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# download binary
BIN_FILE=/tmp/${APP_NAME}
echo "[1/5] Download binary to ${BIN_FILE}"
FILE_URL=${REPO_URL}/releases/download/${VERSION}/${APP_NAME}-${TARGET_TRIPLE}.tar.gz
curl -L ${FILE_URL} | tar xzvf - --no-same-owner -C /tmp/

echo "[2/5] Installing binary to ${BIN_INSTALL_PATH} (requires sudo)."
sudo install -Dm755 "${BIN_FILE}" "${BIN_INSTALL_PATH}"

echo "[3/5] Writing default environment file to ${ENV_FILE} (requires sudo)."
if [[ ! -f "${ENV_FILE}" ]]; then
  sudo tee "${ENV_FILE}" >/dev/null <<'EOF'
# Environment for jelly-fpga-server
# Set OPTIONS to pass flags to the server. Examples:
#   OPTIONS="--port 8051 --verbose 0"
#   OPTIONS="--external --port 8051 --verbose 1"
#   OPTIONS="--external --allow-sudo --port 8051 --verbose 1"
# See: ./jelly-fpga-server --help

OPTIONS="--port 8051 --external --verbose 0"
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
ExecStart=${BIN_INSTALL_PATH} \$OPTIONS
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
