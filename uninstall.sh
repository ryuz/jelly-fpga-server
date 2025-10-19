#!/usr/bin/env bash

set -euo pipefail

# jelly-fpga-server uninstaller
# - Stops and disables the systemd service
# - Removes the systemd service file
# - Optionally removes the binary and environment file

APP_NAME="jelly-fpga-server"
BIN_INSTALL_PATH="/usr/local/bin/${APP_NAME}"
ENV_FILE="/etc/default/${APP_NAME}"
SERVICE_FILE="/etc/systemd/system/${APP_NAME}.service"

confirm() {
  local prompt="$1"
  read -r -p "${prompt} [y/N]: " ans || true
  case "${ans:-}" in
    y|Y|yes|YES) return 0 ;;
    *) return 1 ;;
  esac
}

echo "[1/4] Stopping service if running (requires sudo)."
if systemctl list-unit-files | grep -q "^${APP_NAME}\.service"; then
  sudo systemctl stop "${APP_NAME}.service" || true
  sudo systemctl disable "${APP_NAME}.service" || true
fi

echo "[2/4] Removing systemd service file (requires sudo)."
if [[ -f "${SERVICE_FILE}" ]]; then
  sudo rm -f "${SERVICE_FILE}"
  sudo systemctl daemon-reload
fi

echo "[3/4] Remove installed binary? ${BIN_INSTALL_PATH}"
if [[ -f "${BIN_INSTALL_PATH}" ]]; then
  if confirm "Remove ${BIN_INSTALL_PATH}?"; then
    sudo rm -f "${BIN_INSTALL_PATH}"
  else
    echo "Keeping binary."
  fi
else
  echo "Binary not found; nothing to remove."
fi

echo "[4/4] Remove environment file? ${ENV_FILE}"
if [[ -f "${ENV_FILE}" ]]; then
  if confirm "Remove ${ENV_FILE}?"; then
    sudo rm -f "${ENV_FILE}"
  else
    echo "Keeping environment file."
  fi
else
  echo "Environment file not found; nothing to remove."
fi

echo "Uninstall completed."
