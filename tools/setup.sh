#!/usr/bin/env bash
#
# Reports what this machine needs to work on Vulcan, and installs what it safely
# can.
#
#   ./tools/setup.sh             report only; changes nothing
#   ./tools/setup.sh --install   also install what can be installed without sudo
#
# System packages are never installed for you. The command is printed instead,
# because a build script that runs `sudo` against a package manager is a build
# script that can break a machine it does not understand.
set -uo pipefail
cd "$(dirname "$0")/.."

INSTALL=0
[ "${1:-}" = "--install" ] && INSTALL=1

PLATFORM="$(uname -s)"
missing=0
fixable=0

have()    { command -v "$1" >/dev/null 2>&1; }
ok()      { printf '  \033[32mok\033[0m    %-22s %s\n' "$1" "$2"; }
gap()     { printf '  \033[31mmiss\033[0m  %-22s %s\n' "$1" "$2"; missing=$((missing + 1)); }
todo()    { printf '  \033[33mtodo\033[0m  %-22s %s\n' "$1" "$2"; fixable=$((fixable + 1)); }
note()    { printf '        %-22s %s\n' "" "$1"; }

echo
echo "Vulcan — what this machine needs"
echo "  platform: $PLATFORM $(uname -m)"
echo

# ---- Everywhere --------------------------------------------------------------
echo "Build:"
if have cargo; then
  ok "cargo" "$(cargo --version 2>&1 | cut -d' ' -f1-2)"
else
  gap "cargo" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi
have git && ok "git" "$(git --version | cut -d' ' -f3)" || gap "git" "install git"

# ---- Platform-specific -------------------------------------------------------
if [ "$PLATFORM" = "Darwin" ]; then
  echo
  echo "macOS:"
  if xcode-select -p >/dev/null 2>&1; then
    ok "command line tools" "$(xcode-select -p)"
  else
    gap "command line tools" "xcode-select --install"
  fi

  # The one that is easy to get wrong. GPUI compiles its Metal shaders during
  # the build, and the `metal` compiler ships with full Xcode — the Command Line
  # Tools do not include it. Without this the build fails after several minutes
  # with "xcrun: error: unable to find utility metal".
  if xcrun -f metal >/dev/null 2>&1; then
    ok "metal compiler" "$(xcrun -f metal)"
  else
    gap "metal compiler" "GPUI compiles Metal shaders; this is required"
    note "1. Install Xcode from the App Store. The Command Line Tools do not"
    note "   include the metal compiler, whatever else they provide."
    note "2. sudo xcode-select -s /Applications/Xcode.app/Contents/Developer"
    note "3. xcodebuild -runFirstLaunch"
    note "4. On Xcode 26 the Metal toolchain is a separate download:"
    note "   xcodebuild -downloadComponent metalToolchain"
    note "Then: xcrun -f metal   should print a path."
  fi

  # dnctl and pfctl ship with macOS; note them so the list is complete.
  have dnctl && ok "dnctl" "latency profiles (needs sudo)" || gap "dnctl" "expected on macOS"
  have pfctl && ok "pfctl" "latency profiles (needs sudo)" || gap "pfctl" "expected on macOS"

elif [ "$PLATFORM" = "Linux" ]; then
  echo
  echo "Linux — rendering and capture:"
  have sway && ok "sway" "headless compositor for capture" \
    || gap "sway" "apt install sway"
  have grim && ok "grim" "screenshot via wlr-screencopy" \
    || gap "grim" "apt install grim"
  if [ -f /usr/share/vulkan/icd.d/lvp_icd.json ]; then
    ok "lavapipe" "software Vulkan; GPUI needs a Vulkan device"
  else
    gap "lavapipe" "apt install mesa-vulkan-drivers"
  fi
  have tc && ok "tc" "latency profiles (needs NET_ADMIN)" \
    || gap "tc" "apt install iproute2"

  echo
  echo "Linux — the pinned comparison environment:"
  if have docker || have podman; then
    ok "container runtime" "$(command -v podman || command -v docker)"
  else
    gap "container runtime" "apt install docker.io  — gate 8 compare needs it"
  fi
fi

# ---- Python tooling ----------------------------------------------------------
echo
echo "Python:"
if have python3; then
  ok "python3" "$(python3 --version 2>&1 | cut -d' ' -f2)"

  if python3 -c "import fontTools" >/dev/null 2>&1; then
    ok "fontTools" "rebuilds assets/fonts from the vendored woff2"
  else
    todo "fontTools" "only needed by make fonts"
    if [ "$INSTALL" -eq 1 ]; then
      python3 -m pip install --quiet --user fonttools brotli 2>/dev/null \
        && note "installed" || note "could not install; try: pip install --user fonttools brotli"
    fi
  fi

  if [ -x target/py/bin/pytest ] || [ -x target/py/bin/python ]; then
    ok "pytest venv" "target/py"
  else
    todo "pytest venv" "gate 9's thirty tests run in it"
    if [ "$INSTALL" -eq 1 ]; then
      python3 -m venv target/py >/dev/null 2>&1 \
        && target/py/bin/pip install --quiet pytest >/dev/null 2>&1 \
        && note "created target/py" || note "could not create the venv"
    fi
  fi
else
  gap "python3" "gate 9 and the font build need it"
fi

# ---- Optional ----------------------------------------------------------------
echo
echo "Optional:"
have gh && ok "gh" "$(gh --version | head -1 | cut -d' ' -f3)" \
  || todo "gh" "only for tools/ci.sh; https://cli.github.com"

# ---- Verdict -----------------------------------------------------------------
echo
if [ "$missing" -gt 0 ]; then
  echo "  $missing required, $fixable optional. Install the required ones above."
  [ "$INSTALL" -eq 0 ] && [ "$fixable" -gt 0 ] && echo "  Re-run with --install to set up the optional Python tooling."
  exit 1
fi

if [ "$fixable" -gt 0 ] && [ "$INSTALL" -eq 0 ]; then
  echo "  Everything required is present. $fixable optional item(s); --install sets them up."
else
  echo "  Everything required is present."
fi
echo
echo "  next: make build && make gates"
