#!/usr/bin/env bash
# Captures the shell in each notable state, for people to look at.
#
# These are review material, not gate evidence. The machine's check is
# `gate-fidelity compare`, which runs in the pinned container and compares
# against an approved reference; this script runs a plain headless session on
# whatever host it is on, so its output is not comparable across machines.
# reports/screenshots/ is gitignored for that reason.
set -euo pipefail
cd "$(dirname "$0")/.."

OUT="reports/screenshots"
PREFIX="${1:-F000-engineering-baseline}"

for tool in sway grim; do
  command -v "$tool" >/dev/null || { echo "ERROR: $tool is required" >&2; exit 1; }
done

export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/vulcan-wl}"
mkdir -p "$XDG_RUNTIME_DIR" && chmod 700 "$XDG_RUNTIME_DIR"
# Software Vulkan: these machines have no GPU, and GPUI needs a Vulkan device.
export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.json}"
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
unset DISPLAY

mkdir -p "$OUT"
cargo build --quiet --workspace

# state name -> environment that produces it
STATES=(
  ":"
  "palette:VULCAN_OVERLAY=palette"
  "runconfig:VULCAN_OVERLAY=runconfig"
  "completion-compact:VULCAN_COMPLETION=compact"
  "side-collapsed:VULCAN_SIDE=collapsed"
  "dock-collapsed:VULCAN_DOCK=collapsed"
  "structure:VULCAN_RAIL=structure"
  "commit:VULCAN_RAIL=commit"
  "history:VULCAN_RAIL=history"
  "packs:VULCAN_RAIL=packs"
)

capture() {
  local suffix="$1" env="$2" out
  out="$OUT/$PREFIX${suffix:+-$suffix}.png"

  local config; config="$(mktemp)"
  {
    echo "output HEADLESS-1 mode 1440x900"
    echo "default_border none"
    # Render, wait for the first frame to land, capture, then leave.
    echo "exec sh -c 'cd $PWD && env $env ./target/debug/shell-preview --screenshot \"$out\" >/dev/null 2>&1 & sleep 6; grim -o HEADLESS-1 \"$out\"; swaymsg exit'"
  } > "$config"

  sway -c "$config" >/dev/null 2>&1 || true
  rm -f "$config"

  if [ -s "$out" ]; then
    printf '  %-26s %s\n' "${suffix:-default}" "$out"
  else
    printf '  %-26s FAILED\n' "${suffix:-default}" >&2
    return 1
  fi
}

echo "Capturing ${#STATES[@]} states to $OUT/"
failed=0
for entry in "${STATES[@]}"; do
  capture "${entry%%:*}" "${entry#*:}" || failed=$((failed + 1))
done

echo
[ "$failed" -eq 0 ] && echo "all states captured" || { echo "$failed state(s) failed" >&2; exit 1; }
