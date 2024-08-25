#!/usr/bin/env bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
nix run github:InfiniteCoder01/pixamo -- "$SCRIPT_DIR/player/pose.png" "$SCRIPT_DIR/player/skinmap.png" -s "$SCRIPT_DIR/player/skin.png" -o "$SCRIPT_DIR/player/image.png"
nix run github:InfiniteCoder01/pixamo -- "$SCRIPT_DIR/player/pose.png" "$SCRIPT_DIR/player/skinmap.png" -s "$SCRIPT_DIR/player/skin_flip.png" -o "$SCRIPT_DIR/player/image_flip.png"
nix run github:InfiniteCoder01/pixamo -- "$SCRIPT_DIR/player/pose.png" "$SCRIPT_DIR/player/skinmap.png" -s "$SCRIPT_DIR/player/skin_nowatch.png" -o "$SCRIPT_DIR/player/image_nowatch.png"
