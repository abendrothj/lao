# Conditional Workflows and Multi-Modal Input

This document describes LAO's enhanced capabilities for intelligent workflow orchestration with conditional logic and multi-modal input support.

## Overview

LAO now supports:
- **Conditional Workflow Execution**: Steps can execute based on previous step outcomes
- **Multi-Modal Input Processing**: Support for audio, image, video, and file inputs beyond text
- **Enhanced Error Handling**: Conditional fallbacks and branching logic
- **Real-World Logic**: Workflows that adapt to runtime conditions

## Conditional Workflow Syntax

### Basic Condition Structure

```yaml
steps:
  - run: PluginName
    input: "data"
    condition:
      condition_type: OutputContains
      field: "previous_step_id"
      operator: Contains
      value: "success"
```

### Condition Types

| Type | Description | Example Use Case |
|------|-------------|------------------|
| `OutputContains` | Check if step output contains text | File type detection |
| `OutputEquals` | Exact output matching | Status verification |
| `StatusEquals` | Check step execution status | Success/failure branching |
| `ErrorContains` | Check error message content | Error-specific handling |
| `PreviousStepStatus` | Check last executed step status | Sequential decision making |

### Condition Operators

| Operator | Description | Works With |
|----------|-------------|------------|
| `Equals` | Exact match | All types |
| `NotEquals` | Not equal | All types |
| `Contains` | Substring match | Text/Output |
| `NotContains` | Does not contain | Text/Output |
| `GreaterThan` | Numeric comparison | Numbers (future) |
| `LessThan` | Numeric comparison | Numbers (future) |

## Multi-Modal Input Types

### Supported Input Types

| Type | Extensions | Use Cases |
|------|------------|-----------|
| `Audio` | .wav, .mp3, .flac, .m4a | Speech transcription, audio analysis |
| `Image` | .jpg, .jpeg, .png, .gif, .bmp | Image recognition, OCR |
| `Video` | .mp4, .avi, .mov, .mkv, .webm | Video processing, frame extraction |
| `File` | Any file type | Document processing, data analysis |
| `Text` | .txt, .md, .json, .yaml, .yml | Text processing, configuration |

### Multi-Modal Workflow Example

```yaml
workflow: "Multi-Media Processing"
steps:
  - run: ContentDetector
    input: "media_file.mp4"
    input_type: Video
    
  - run: VideoProcessor
    input_from: ContentDetector
    condition:
      condition_type: OutputContains
      field: ContentDetector
      operator: Contains
      value: "video"
      
  - run: AudioExtractor
    input_from: VideoProcessor
    
  - run: WhisperPlugin
    input_from: AudioExtractor
    input_type: Audio
```

## Complete Examples

### 1. Smart File Processing with Conditional Branching

```yaml
workflow: "Smart Document Processing"
steps:
  - run: FileTypeDetector
    input: "document.pdf"
    input_type: File
    
  - run: PDFProcessor
    input_from: FileTypeDetector
    condition:
      condition_type: OutputContains
      field: FileTypeDetector
      operator: Contains
      value: "pdf"
      
  - run: ImageProcessor
    input_from: FileTypeDetector
    condition:
      condition_type: OutputContains
      field: FileTypeDetector
      operator: Contains
      value: "image"
      
  - run: TextProcessor
    input_from: FileTypeDetector
    condition:
      condition_type: OutputContains
      field: FileTypeDetector
      operator: Contains
      value: "text"
      
  - run: SummarizerPlugin
    depends_on: ["PDFProcessor", "ImageProcessor", "TextProcessor"]
```

### 2. Error Handling with Fallbacks

```yaml
workflow: "Robust Image Analysis"
steps:
  - run: ImageAnalyzer
    input: "photo.jpg"
    input_type: Image
    retries: 2
    retry_delay: 1000
    
  - run: EnhancedProcessor
    input_from: ImageAnalyzer
    condition:
      condition_type: PreviousStepStatus
      field: ""
      operator: Equals
      value: "success"
      
  - run: FallbackProcessor
    input: "Using basic image processing"
    condition:
      condition_type: PreviousStepStatus
      field: ""
      operator: Equals
      value: "error"
      
  - run: ResultFormatter
    depends_on: ["EnhancedProcessor", "FallbackProcessor"]
```

### 3. Voice Command Workflow

```yaml
workflow: "Voice Command Processing"
steps:
  - run: WhisperPlugin
    input: "voice_command.wav"
    input_type: Audio
    
  - run: IntentClassifier
    input_from: WhisperPlugin
    
  - run: TaskExecutor
    input_from: IntentClassifier
    condition:
      condition_type: OutputContains
      field: IntentClassifier
      operator: Contains
      value: "execute"
      
  - run: InformationProvider
    input_from: IntentClassifier
    condition:
      condition_type: OutputContains
      field: IntentClassifier
      operator: Contains
      value: "query"
      
  - run: ConfirmationResponder
    depends_on: ["TaskExecutor", "InformationProvider"]
```

## Execution Behavior

### Step Statuses

| Status | Description | When Used |
|--------|-------------|-----------|
| `pending` | Waiting to execute | Initial state |
| `running` | Currently executing | During execution |
| `success` | Completed successfully | Normal completion |
| `error` | Failed execution | Error occurred |
| `cache` | Result from cache | Cache hit |
| `skipped` | Condition not met | Conditional skip |

### Conditional Execution Flow

1. **Condition Evaluation**: Before executing a step, LAO evaluates its condition (if present)
2. **Skip Decision**: If condition fails, step is marked as "skipped" and execution continues
3. **Dependencies**: Skipped steps don't block dependent steps unless explicitly required
4. **Status Tracking**: All step statuses are tracked for subsequent condition evaluation

## CLI Usage

### Running Conditional Workflows

```bash
cd core
../target/release/lao-cli run ../workflows/conditional_example.yaml
```

### Validating Conditional Workflows

```bash
cd core
../target/release/lao-cli validate ../workflows/conditional_example.yaml
```

### Testing with Enhanced Prompt Library

```bash
cd core
../target/release/lao-cli validate-prompts
```

## Best Practices

### 1. Condition Design
- Use specific field references when possible
- Design conditions that are deterministic
- Consider edge cases and fallback scenarios

### 2. Error Handling
- Always provide fallback paths for critical workflows
- Use retries for transient failures
- Log conditions for debugging

### 3. Multi-Modal Processing
- Validate file types before processing
- Handle different media formats gracefully
- Consider file size limits and processing time

### 4. Performance Considerations
- Conditional steps can improve efficiency by skipping unnecessary work
- Cache results when appropriate
- Use parallel execution where dependencies allow

## Troubleshooting

### Common Issues

1. **Condition Never Met**: Check field names and expected values
2. **Unexpected Skips**: Verify condition logic and previous step outputs
3. **Type Mismatches**: Ensure input/output types are compatible
4. **File Not Found**: Verify file paths and permissions for multi-modal inputs

### Debugging Tips

```bash
# Enable debug logging
export RUST_LOG=debug

# Run with verbose output
../target/release/lao-cli run ../workflows/conditional_example.yaml --verbose
```

## Integration with UI

The LAO UI now supports:
- Visual indication of conditional steps
- Real-time status updates for skipped steps
- File upload for multi-modal inputs
- Drag-and-drop workflow creation with conditions

## Future Enhancements

Planned improvements:
- Visual condition builder in UI
- More condition types (numeric comparisons, regex matching)
- Advanced branching (loops, parallel conditions)
- Template workflows for common patterns