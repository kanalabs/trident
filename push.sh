#!/bin/bash

# Variables
DOCKER_USERNAME="kanalabs"
DOCKER_IMAGE_NAME="trident"
DOCKER_TAG="latest"

# Step 1: Build the Docker image
echo "Building Docker image..."
docker build -t ${DOCKER_USERNAME}/${DOCKER_IMAGE_NAME}:${DOCKER_TAG} --platform linux/amd64 .

# Step 2: Push the Docker image to Docker Hub
echo "Pushing Docker image to Docker Hub..."
docker push ${DOCKER_USERNAME}/${DOCKER_IMAGE_NAME}:${DOCKER_TAG}

echo "Docker image pushed successfully!"