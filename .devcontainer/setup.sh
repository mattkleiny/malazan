#!/usr/bin/env bash
set -euo pipefail

SUDO=""
if [ "$(id -u)" != "0" ]; then SUDO="sudo"; fi
${SUDO} apt-get update -q && ${SUDO} apt-get install -y -q curl unzip

GODOT_VERSION="4.8"
GODOT_RELEASE="dev7"
GODOT_ARCHIVE="Godot_v${GODOT_VERSION}-${GODOT_RELEASE}_linux.x86_64.zip"
# GODOT_URL="https://github.com/godotengine/godot/releases/download/${GODOT_VERSION}-${GODOT_RELEASE}/${GODOT_ARCHIVE}"
GODOT_URL="https://downloads.godotengine.org/?version=${GODOT_VERSION}&flavor=${GODOT_RELEASE}&slug=linux.x86_64.zip&platform=linux.64"
INSTALL_DIR="/usr/local/bin"

echo "Downloading Godot ${GODOT_VERSION}..."
curl -fsSL "${GODOT_URL}" -o "/tmp/${GODOT_ARCHIVE}"

echo "Installing Godot ${GODOT_VERSION}..."
unzip -q "/tmp/${GODOT_ARCHIVE}" -d /tmp/godot
${SUDO} install -m 755 "/tmp/godot/Godot_v${GODOT_VERSION}-${GODOT_RELEASE}_linux.x86_64" "${INSTALL_DIR}/godot4"

rm -rf "/tmp/${GODOT_ARCHIVE}" /tmp/godot

echo "Godot ${GODOT_VERSION} installed to ${INSTALL_DIR}/godot4"
godot4 --version

if [ -f .gitmodules ]; then
  echo "Initializing git submodules..."
  git submodule update --init --recursive
fi
