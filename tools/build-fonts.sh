#!/usr/bin/env bash
# Regenerates assets/fonts/ from the woff2 the prototype vendors.
#
# The text shaper cannot read woff2, so the faces are converted to TrueType and
# embedded with include_bytes!. The outputs are committed because the crate does
# not compile without them; this script exists so they are reproducible rather
# than mysterious, and so a prototype that changes its typefaces can be followed
# with one command.
set -euo pipefail
cd "$(dirname "$0")/.."

command -v python3 >/dev/null || { echo "python3 required" >&2; exit 1; }
python3 -c "import fontTools" 2>/dev/null || { echo "pip install fonttools brotli" >&2; exit 1; }

python3 - <<'PY'
from fontTools.ttLib import TTFont
from pathlib import Path

# source woff2 -> embedded ttf
FACES = {
    "mockups/fonts/inter-latin-400.woff2": "assets/fonts/inter-latin-400.ttf",
    "mockups/fonts/inter-latin-500.woff2": "assets/fonts/inter-latin-500.ttf",
    "mockups/fonts/inter-latin-600.woff2": "assets/fonts/inter-latin-600.ttf",
    "mockups/fonts/inter-latin-700.woff2": "assets/fonts/inter-latin-700.ttf",
    "mockups/fonts/jetbrains-mono-latin-400.woff2": "assets/fonts/jetbrains-mono-400.ttf",
    "mockups/fonts/jetbrains-mono-latin-500.woff2": "assets/fonts/jetbrains-mono-500.ttf",
}

Path("assets/fonts").mkdir(parents=True, exist_ok=True)
for src, dst in FACES.items():
    font = TTFont(src)
    font.flavor = None
    font.save(dst)
    print(f"{dst:44} {Path(dst).stat().st_size // 1024:>4} KB  {font['name'].getDebugName(1)}")

# Phosphor ships only as a ttf in the icon bundle.
icon = Path("mockups/icons/ttf/Phosphor.ttf")
if icon.exists():
    out = Path("assets/fonts/Phosphor.ttf")
    out.write_bytes(icon.read_bytes())
    print(f"{out!s:44} {out.stat().st_size // 1024:>4} KB  Phosphor (copied)")
PY
