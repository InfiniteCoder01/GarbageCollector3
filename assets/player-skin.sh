#!/usr/bin/env bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
nix run github:InfiniteCoder01/pixamo -- "$SCRIPT_DIR/player/pose.png" "$SCRIPT_DIR/player/skinmap.png" -s "$SCRIPT_DIR/player/skin.png" -o "$SCRIPT_DIR/player/image.png"
