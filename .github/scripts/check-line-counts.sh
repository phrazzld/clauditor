#!/bin/bash
# .github/scripts/check-line-counts.sh

# This script checks for files that exceed a maximum line count to encourage
# modularity and maintainability.

MAX_LINES=300
EXIT_CODE=0

# Find all .rs files in the src directory
FILES=$(find src -name "*.rs")

echo "Checking line counts for .rs files in src/..."

for FILE in $FILES; do
  LINES=$(wc -l < "$FILE" | xargs) # xargs trims whitespace
  if [ "$LINES" -gt "$MAX_LINES" ]; then
    echo "Error: File '$FILE' has $LINES lines, which exceeds the limit of $MAX_LINES."
    EXIT_CODE=1
  else
    echo "✔ $FILE ($LINES lines)"
  fi
done

if [ "$EXIT_CODE" -ne 0 ]; then
  echo "Line count check failed. Please refactor the oversized files."
fi

exit $EXIT_CODE
