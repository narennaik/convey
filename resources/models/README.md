# Whisper Models

This directory contains the Whisper model files used for transcription.

## Download the Model

The model files are not included in the repository due to their large size (141MB).

**To download the base model, run:**

```bash
./scripts/download_model.sh
```

This will download `ggml-base.bin` from the official Whisper.cpp repository.

## Manual Download

Alternatively, download manually:

```bash
curl -L -o resources/models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

## Other Models

You can use different Whisper models for different accuracy/speed tradeoffs:

- **tiny** (~75MB) - Fastest, least accurate
- **base** (~141MB) - **Default**, good balance
- **small** (~466MB) - Better accuracy
- **medium** (~1.5GB) - High accuracy
- **large** (~2.9GB) - Best accuracy

Download other models:
```bash
# Example: Download medium model
curl -L -o resources/models/ggml-medium.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin
```

Then update `src/whisper.rs` to use the new model name.
