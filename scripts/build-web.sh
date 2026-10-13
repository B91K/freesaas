#!/usr/bin/env bash
# Build the static site: the hub (web/) plus the editors' WASM builds.
#
#   scripts/build-web.sh [OUT_DIR] [EDITOR...]
#
# OUT_DIR defaults to dist/site. EDITORs default to all three; pass names to
# build a subset (CI builds each in its own job). `hub` alone copies only the
# hub. Needs: the wasm32-unknown-unknown target and trunk on PATH.
#
# FREESAAS_PUBLIC_CONFIG, if set, is written to OUT_DIR/config.json. It is
# served to every visitor: put only public values in it, never secrets.
set -euo pipefail
cd "$(dirname "$0")/.."
root=$PWD
out=$(realpath -m "${1:-dist/site}")
shift || true
editors=("$@")
[ ${#editors[@]} -eq 0 ] && editors=(hub wordcraft gridcraft deckcraft)

mkdir -p "$out"
for e in "${editors[@]}"; do
  case "$e" in
    hub)
      cp -r web/. "$out/"
      rm -f "$out/README.md"
      if [ -n "${FREESAAS_PUBLIC_CONFIG:-}" ]; then
        printf '%s\n' "$FREESAAS_PUBLIC_CONFIG" > "$out/config.json"
      fi
      ;;
    # Ours: WordCraft wired to the hub (editors/wordcraft-embed).
    wordcraft) dir=editors/wordcraft-embed ;;
    # Upstream as-is until their hub bridge is written.
    gridcraft | deckcraft) dir=upstream/$e/apps/$e-web ;;
    *) echo "unknown editor: $e" >&2; exit 1 ;;
  esac
  [ "$e" = hub ] && continue
  echo "==> building $e"
  # Separate target dirs keep each workspace's incremental cache usable.
  (cd "$dir" && CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/web}/$e" \
    trunk build --release --dist "$out/editors/$e")
done
echo "Site in $out"
