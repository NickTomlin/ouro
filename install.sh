#!/usr/bin/env bash
set -euo pipefail

REPO="NickTomlin/ouro"
BIN_DIR="${OURO_INSTALL_DIR:-/usr/local/bin}"
BIN_NAME="ouro"

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64) echo "ouro-linux-x86_64" ;;
        *) echo "Unsupported Linux architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64)  echo "ouro-macos-x86_64" ;;
        arm64)   echo "ouro-macos-aarch64" ;;
        *) echo "Unsupported macOS architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "Unsupported OS: $os. Download manually from https://github.com/$REPO/releases" >&2
      exit 1
      ;;
  esac
}

ASSET="$(detect_target)"
VERSION="${OURO_VERSION:-latest}"

if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/$REPO/releases/latest/download/$ASSET"
else
  URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
fi

echo "Downloading $ASSET from $URL ..."
curl -sSfL "$URL" -o "$BIN_DIR/$BIN_NAME"
chmod +x "$BIN_DIR/$BIN_NAME"
echo "Installed ouro to $BIN_DIR/$BIN_NAME"
