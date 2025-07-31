# WhisperPlugin

A plugin that transcribes audio files to text using a local Whisper model.

## Input
- `audio_file` (string): Path to the audio file to transcribe.

## Output
- (string): The transcribed text.

## Example Workflow
```yaml
workflow: "Audio Transcription"
steps:
  - run: Whisper
    audio_file: "meeting.wav"
```

## Usage
Reference the plugin by name in your workflow YAML as shown above. The output can be used as input for downstream plugins (e.g., Summarizer). 