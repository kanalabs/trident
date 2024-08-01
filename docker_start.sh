#!/bin/bash

# Define variables
IMAGE_NAME="trident_app"
CONTAINER_NAME="trident_container"

# Define the path to the custom config file, if any
CUSTOM_CONFIG_PATH="./config.toml"

# Build the Docker image
echo "Building Docker image..."
docker build -t $IMAGE_NAME .

# Check if an old container with the same name exists and remove it
if [ "$(docker ps -aq -f name=$CONTAINER_NAME)" ]; then
    echo "Removing old container..."
    docker rm -f $CONTAINER_NAME
fi

# Check if the custom config file exists
if [ -f "$CUSTOM_CONFIG_PATH" ]; then
    echo "Custom config.toml found. Using custom configuration."
    CONFIG_OPTION="-v $CUSTOM_CONFIG_PATH:/app/config.toml"
else
    echo "Custom config.toml not found. Using default configuration."
    CONFIG_OPTION=""
fi

# Run the Docker container
echo "Starting Docker container..."
docker run -d --name $CONTAINER_NAME -p 3001:3001 $CONFIG_OPTION $IMAGE_NAME

echo "Docker container started successfully."
