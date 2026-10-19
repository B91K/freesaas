#!/usr/bin/env bash
# Move the editor submodules to the latest upstream commit and show what
# changed. Review, run the checks, then commit the new pointers.
set -euo pipefail
cd "$(dirname "$0")/.."
for app in wordcraft gridcraft deckcraft; do
  old=$(git -C "upstream/$app" rev-parse --short HEAD)
  git submodule update --remote --depth 1 "upstream/$app"
  new=$(git -C "upstream/$app" rev-parse --short HEAD)
  echo "$app: $old -> $new"
done
git status --short upstream
