#!/bin/bash
# Script to run the clockradio TUI application

echo "Building and running clockradio..."

# Check if we're in a Nix environment and use appropriate build command
if command -v nix-shell &> /dev/null; then
    nix-shell -p gcc rustc cargo --run "cargo run --release"
else
    cargo run --release
fi