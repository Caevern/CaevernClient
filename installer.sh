#!/usr/bin/env bash
set -euo pipefail

BIN="$(realpath ./target/release/Caevern)"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"

install -Dm644 assets/icons/icon.png \
    "$DATA_HOME/icons/hicolor/256x256/apps/caevern.png"

mkdir -p "$DATA_HOME/applications"

cat > "$DATA_HOME/applications/caevern.desktop" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Caevern
Comment=Caevern VR Client
Exec=$BIN
Icon=caevern
Terminal=false
Categories=Game;
StartupNotify=true
EOF
