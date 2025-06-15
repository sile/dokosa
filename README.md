dokosa
======

[![dokosa](https://img.shields.io/crates/v/dokosa.svg)](https://crates.io/crates/dokosa)
[![Actions Status](https://github.com/sile/dokosa/workflows/CI/badge.svg)](https://github.com/sile/dokosa/actions)
![License](https://img.shields.io/crates/l/dokosa)

A command-line semantic search tool that indexes and searches local Git repositories using vector embeddings.

"dokosa" means "where is it?" in Japanese.

## Features

- **Semantic indexing**: Uses OpenAI embeddings to create searchable vector representations of code
- **Git integration**: Automatically tracks repository commits and file changes
- **Flexible filtering**: Include/exclude files using glob patterns
- **Chunked processing**: Splits large files into overlapping chunks for better search granularity
- **Similarity search**: Find code snippets based on semantic meaning, not just keyword matching

## Installation

```console
$ cargo install dokosa
```

## Quick Start

```console
# Set your OpenAI API key
$ export OPENAI_API_KEY="your-api-key-here"

# Set the index file path
$ export DOKOSA_INDEX_FILE="$HOME/.dokosa"

# Add a repository to the index
$ dokosa add /path/to/your/repo

# Search for code semantically
$ echo "function to parse JSON" | dokosa search

# Sync repositories with latest commits
$ dokosa sync
```

## Commands

- `add` - Index a Git repository for semantic search
- `search` - Find semantically similar code chunks
- `list` - Show all indexed repositories
- `sync` - Update repositories with latest changes
- `remove` - Remove a repository from the index

Run `dokosa <command> --help` for detailed options.
