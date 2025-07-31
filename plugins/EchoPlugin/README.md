# EchoPlugin

A simple plugin that echoes the input text. Useful for testing and demonstration purposes.

## Input
- `input` (string): The text to echo.

## Output
- (string): The same text as input.

## Example Workflow
```yaml
workflow: "Echo Test"
steps:
  - run: Echo
    input: "Hello, LAO!"
```

## Usage
Reference the plugin by name in your workflow YAML as shown above. 