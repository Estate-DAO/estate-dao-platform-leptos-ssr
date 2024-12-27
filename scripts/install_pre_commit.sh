#!/bin/bash

# Define the source and destination paths
SOURCE="./scripts/pre-commit"
DESTINATION=".git/hooks/pre-commit"

# Check if the source file exists
if [ ! -f "$SOURCE" ]; then
  echo "Error: Source file '$SOURCE' does not exist."
  exit 1
fi

chmod +x "$SOURCE"

# Create a symlink for the pre-commit hook
ln -sf "$(pwd)/$SOURCE" "$DESTINATION"

echo "Symlink created: $SOURCE -> $DESTINATION"
