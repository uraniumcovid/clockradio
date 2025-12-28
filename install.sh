#!/bin/bash

set -e

echo "Installing clockradio..."

# Build the project in release mode
echo "Building clockradio..."
cargo build --release

# Get the target directory and binary path
TARGET_DIR="target/release"
BINARY_NAME="clockradio"
BINARY_PATH="$PWD/$TARGET_DIR/$BINARY_NAME"

# Create a local bin directory if it doesn't exist
LOCAL_BIN="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN"

# Copy the binary to the local bin directory
echo "Installing binary to $LOCAL_BIN..."
cp "$BINARY_PATH" "$LOCAL_BIN/"

# Make sure it's executable
chmod +x "$LOCAL_BIN/$BINARY_NAME"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$LOCAL_BIN:"* ]]; then
    echo "Adding $LOCAL_BIN to PATH..."
    
    # Determine which shell config file to update
    if [ -n "$ZSH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.bashrc"
    else
        SHELL_CONFIG="$HOME/.profile"
    fi
    
    # Add to PATH in shell config
    echo "" >> "$SHELL_CONFIG"
    echo "# Added by clockradio installer" >> "$SHELL_CONFIG"
    echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> "$SHELL_CONFIG"
    
    echo "Added PATH export to $SHELL_CONFIG"
    echo "Please run 'source $SHELL_CONFIG' or restart your terminal to use the 'clockradio' command."
else
    echo "PATH already includes $LOCAL_BIN"
fi

echo "Installation complete!"
echo "You can now run 'clockradio' from anywhere in your terminal."