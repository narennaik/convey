#!/bin/bash
set -e

MODEL_DIR="resources/models"
MODEL_FILE="$MODEL_DIR/ggml-base.bin"
MODEL_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"

echo "Downloading Whisper base model..."

# Create directory if it doesn't exist
mkdir -p "$MODEL_DIR"

# Check if model already exists
if [ -f "$MODEL_FILE" ]; then
    echo "✅ Model already exists at $MODEL_FILE"
    echo "   Size: $(du -h "$MODEL_FILE" | cut -f1)"
    exit 0
fi

# Download the model
echo "Downloading from $MODEL_URL..."
echo "This may take a few minutes (model is ~141MB)..."

curl -L -o "$MODEL_FILE" "$MODEL_URL" --progress-bar

if [ -f "$MODEL_FILE" ]; then
    echo ""
    echo "✅ Model downloaded successfully!"
    echo "   Location: $MODEL_FILE"
    echo "   Size: $(du -h "$MODEL_FILE" | cut -f1)"
else
    echo "❌ Failed to download model"
    exit 1
fi
