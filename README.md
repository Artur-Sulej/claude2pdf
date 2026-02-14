# claude2pdf

A command-line tool to convert Claude Code JSONL conversation logs into beautiful, syntax-highlighted PDFs.

## Features

- **Automatic Formatting**: Automatically extracts assistant and user roles from Claude JSONL exports.
- **Syntax Highlighting**: Uses `syntect` to provide high-quality code highlighting for a wide range of programming languages.
- **Modern PDF Output**: Generates clean, readable PDFs via **Google Chrome's** headless engine.
- **Smart Defaults**: Automatically names output files based on input filenames.

## Prerequisites

This tool requires **Google Chrome** to be installed on your system. It uses Chrome's headless mode to generate high-quality PDFs.
- **macOS**: Installed at `/Applications/Google Chrome.app` (default).

## Installation

### From Source
To install `claude2pdf` as a global command:

```bash
git clone <repository-url>
cd claude2pdf
cargo install --path .
```

## Usage

### Basic Usage
Convert a JSONL file to a PDF with the same name:
```bash
claude2pdf conversation.jsonl
# Creates conversation.pdf
```

### Custom Output Path
```bash
claude2pdf conversation.jsonl -o my_report.pdf
```

### Development Mode
Run without installing:
```bash
cargo run -- conversation.jsonl
```

## How it Works

1. **Extraction**: Parses the JSONL file to extract the text content of the conversation.
2. **Markdown Conversion**: Wraps the content in Markdown formatting.
3. **Highlighting**: Identifies code blocks and applies syntax highlighting using the `base16-ocean.dark` theme.
4. **HTML Rendering**: Converts the Markdown (+ highlighting) into HTML.
5. **PDF Generation**: Uses **Google Chrome** to render the final PDF document.
