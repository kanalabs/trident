#!/bin/sh
# Check if config.toml is present, otherwise use example.config.toml
CONFIG_FILE="/app/config.toml"
DEFAULT_CONFIG_FILE="/app/example.config.toml"

if [ -f "$CONFIG_FILE" ]; then
    echo "Using provided config.toml"
else
    echo "Using default example.config.toml"
    CONFIG_FILE="$DEFAULT_CONFIG_FILE"
fi

# Run the application with the selected configuration file
./trident --config "$CONFIG_FILE"