#!/usr/bin/env bash
#
# Runs on a Mac the checks that cannot be run anywhere else, and writes one JSON
# file per check to reports/macos/ for review elsewhere.
#
#   ./tools/macos-verify.sh            the checks that need no privilege
#   ./tools/macos-verify.sh --sudo     also the latency profiles, which need it
#
# What it needs, all of which it checks before doing any work:
#
#   * a Mac with Apple Silicon — an Intel Mac cannot answer the question this
#     script exists to ask
#   * Xcode Command Line Tools: `xcode-select --install`. Provides clang for the
#     `cc` crate, libclang for `bindgen`, and the SDK holding Metal, which is how
#     GPUI presents on this platform
#   * rustup, which installs the toolchain rust-toolchain.toml pins by itself
#   * roughly 10 GB free, most of it the build
#   * a logged-in desktop session for the check that opens a window; over SSH
#     there is no window server and that one check fails
#   * sudo, only with --sudo
#
# Written for the bash macOS ships, which is 3.2: no associative arrays, no
# ${var,,}. Nothing here installs anything or changes any setting that outlives
# the run.
set -uo pipefail
cd "$(dirname "$0")/.."

OUT="reports/macos"
WITH_SUDO=0
[ "${1:-}" = "--sudo" ] && WITH_SUDO=1

mkdir -p "$OUT"

# One stamp for the whole run, so every file from a single run groups together
# and successive runs accumulate rather than overwrite. Colons are left out
# because they travel badly through filesystems and URLs.
RUN_AT=$(date -u +%Y%m%dT%H%M%SZ)

passed=0
failed=0
skipped=0

# Minimal JSON string escaper: backslash, quote, tab, and newlines.
esc() {
  printf '%s' "$1" | awk '
    BEGIN { ORS = "" }
    {
      gsub(/\\/, "\\\\");
      gsub(/"/, "\\\"");
      gsub(/\t/, "\\t");
      if (NR > 1) print "\\n";
      print;
    }'
}

# record <title> <status> <summary> <detail> [extra-json]
record() {
  title="$1"; status="$2"; summary="$3"; detail="$4"; extra="${5:-}"
  file="$OUT/verification-$title-$status-$RUN_AT.json"
  {
    printf '{\n'
    printf '  "check": "%s",\n' "$(esc "$title")"
    printf '  "status": "%s",\n' "$status"
    printf '  "summary": "%s",\n' "$(esc "$summary")"
    printf '  "recorded_at": "%s",\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf '  "run": "%s",\n' "$RUN_AT"
    printf '  "commit": "%s",\n' "$(git rev-parse HEAD 2>/dev/null || echo unknown)"
    printf '  "host": { "arch": "%s", "os": "%s" },\n' "$(uname -m)" "$(sw_vers -productVersion 2>/dev/null || echo unknown)"
    [ -n "$extra" ] && printf '  %s,\n' "$extra"
    printf '  "detail": "%s"\n' "$(esc "$detail")"
    printf '}\n'
  } > "$file"

  case "$status" in
    PASS) passed=$((passed+1)); printf '  \033[32mPASS\033[0m  %-22s %s\n' "$title" "$summary" ;;
    FAIL) failed=$((failed+1)); printf '  \033[31mFAIL\033[0m  %-22s %s\n' "$title" "$summary" ;;
    *)    skipped=$((skipped+1)); printf '  ----  %-22s %s\n' "$title" "$summary" ;;
  esac
}

# run <title> <summary-on-pass> <command...>
run_check() {
  title="$1"; shift
  ok_summary="$1"; shift
  output=$("$@" 2>&1)
  code=$?
  if [ $code -eq 0 ]; then
    record "$title" PASS "$ok_summary" "$output"
  else
    record "$title" FAIL "exit $code" "$output"
  fi
  return $code
}

echo
echo "Vulcan — macOS verification"
echo "  repository: $(pwd)"
echo "  commit:     $(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
echo

# ---- Before doing any work ---------------------------------------------------
# A missing prerequisite should cost a second, not eight minutes of compiling.
missing=0
need() {
  printf '  %-34s %s\n' "$1" "$2"
  missing=$((missing + 1))
}

echo "Prerequisites:"

if [ "$(uname -s)" != "Darwin" ]; then
  need "macOS" "this is $(uname -s); the checks below only mean something on a Mac"
elif [ "$(uname -m)" != "arm64" ]; then
  need "Apple Silicon" "this is $(uname -m); an Intel Mac cannot answer the topology question"
else
  printf '  %-34s %s\n' "macOS on Apple Silicon" "$(sw_vers -productVersion), $(uname -m)"
fi

if [ "$(uname -s)" = "Darwin" ]; then
  if xcode-select -p >/dev/null 2>&1; then
    printf '  %-34s %s\n' "Xcode Command Line Tools" "$(xcode-select -p)"
  else
    need "Xcode Command Line Tools" "run: xcode-select --install"
  fi
fi

if command -v rustup >/dev/null 2>&1; then
  printf '  %-34s %s\n' "rustup" "$(rustup --version 2>&1 | head -1)"
elif command -v cargo >/dev/null 2>&1; then
  printf '  %-34s %s\n' "cargo (no rustup)" "$(cargo --version 2>&1)"
else
  need "rustup" "install from https://rustup.rs"
fi

free_gb=$(df -g . 2>/dev/null | awk 'NR==2 {print $4}')
if [ -n "${free_gb:-}" ] && [ "$free_gb" -lt 10 ] 2>/dev/null; then
  need "10 GB free" "${free_gb} GB available; the build alone is most of it"
else
  printf '  %-34s %s\n' "disk" "${free_gb:-?} GB free"
fi

if [ "$WITH_SUDO" -eq 1 ] && ! sudo -n true 2>/dev/null; then
  printf '  %-34s %s\n' "sudo" "will prompt"
fi

if [ "$missing" -gt 0 ]; then
  echo
  echo "  $missing prerequisite(s) missing; nothing was run."
  exit 1
fi
echo

# ---- 1. The machine ----------------------------------------------------------
# The open question the whole budget gate rests on: does this Mac report
# performance and efficiency cores separately? A runner that does not cannot
# stand in for the baseline, whatever its core count.
p_cores=$(sysctl -n hw.perflevel0.logicalcpu 2>/dev/null || echo "")
e_cores=$(sysctl -n hw.perflevel1.logicalcpu 2>/dev/null || echo "")
logical=$(sysctl -n hw.logicalcpu 2>/dev/null || echo 0)
mem_bytes=$(sysctl -n hw.memsize 2>/dev/null || echo 0)
mem_gb=$((mem_bytes / 1024 / 1024 / 1024))
chip=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown)

topology_json=$(printf '"machine": { "chip": "%s", "logical_cpu": %s, "performance_cores": %s, "efficiency_cores": %s, "memory_gb": %s }' \
  "$(esc "$chip")" "${logical:-0}" "${p_cores:-0}" "${e_cores:-0}" "$mem_gb")

if [ -z "$p_cores" ]; then
  record "topology" FAIL "no hw.perflevel0.logicalcpu; asymmetry not observable" \
    "This machine does not report performance levels, so the Apple Silicon runner cannot describe its own topology." "$topology_json"
elif [ -z "$e_cores" ] || [ "$e_cores" = "0" ]; then
  record "topology" FAIL "${p_cores}P + 0E — uniform cores, not the baseline's split" \
    "$chip reports $p_cores performance cores and no efficiency cores. The baseline is 2P + 4E." "$topology_json"
else
  record "topology" PASS "${p_cores}P + ${e_cores}E, ${mem_gb} GB" \
    "$chip" "$topology_json"
fi

# ---- 2. Toolchain ------------------------------------------------------------
if command -v cargo >/dev/null 2>&1; then
  record "toolchain" PASS "$(rustc --version 2>&1), $(uname -m)" "$(cargo --version 2>&1)"
else
  record "toolchain" FAIL "cargo not found" "Install Rust: https://rustup.rs"
  echo; echo "  cannot continue without a toolchain"; exit 1
fi

# ---- 3. It builds for arm64 macOS --------------------------------------------
# GPUI presents through Metal here and Vulkan on Linux, so this is the first
# real check that the product compiles for the platform it claims to target.
run_check "build" "workspace compiles" cargo build --workspace --quiet

# ---- 4. Tests ----------------------------------------------------------------
test_out=$(cargo test --workspace 2>&1)
test_code=$?
counts=$(printf '%s' "$test_out" | awk '/^test result/{p+=$4; f+=$6} END{printf "%d passed, %d failed", p, f}')
if [ $test_code -eq 0 ]; then
  record "tests" PASS "$counts" "$test_out"
else
  record "tests" FAIL "$counts" "$test_out"
fi

# ---- 5. Architectural boundaries ---------------------------------------------
run_check "boundary" "gate 1 passes" cargo run --quiet -p gate-boundary

# ---- 6. The shell resolves without a display ---------------------------------
run_check "shell-smoke" "layout resolves from tokens" \
  cargo run --quiet -p shell-preview -- --smoke

# ---- 7. The shell renders, and measures itself -------------------------------
# This opens a real window. It needs a logged-in session, not just SSH.
measure_json="$OUT/measurements-$RUN_AT.json"
measure_out=$(cargo run --quiet -p shell-preview -- --measure "$measure_json" 2>&1)
if [ -s "$measure_json" ]; then
  record "shell-render" PASS "rendered and reported its own timings" "$measure_out" \
    "\"measurements_file\": \"$(esc "$measure_json")\""
else
  record "shell-render" FAIL "no measurements written" \
    "$measure_out

If this says the window could not open, run it from a logged-in desktop session
rather than over SSH: GPUI needs a window server."
fi

# ---- 8. Budgets on the authoritative runner ----------------------------------
budget_out=$(cargo run --quiet -p gate-budget -- --runner apple-silicon --rtt 0 \
  --report "$OUT/budgets-apple-silicon-$RUN_AT.json" 2>&1)
budget_code=$?
case $budget_code in
  0) record "budgets" PASS "every budget met at 0ms round trip" "$budget_out" ;;
  2) record "budgets" FAIL "a budget was exceeded" "$budget_out" ;;
  *) record "budgets" FAIL "could not judge (exit $budget_code)" "$budget_out" ;;
esac

# ---- 9. Latency injection, which needs sudo ----------------------------------
# The dnctl and pfctl path has never been run on a Mac. It is the piece most
# likely to be wrong.
if [ "$WITH_SUDO" -eq 1 ]; then
  lat_out=$(sudo cargo run --quiet -p gate-budget -- --runner apple-silicon --rtt 10 \
    --report "$OUT/budgets-apple-silicon-10ms-$RUN_AT.json" 2>&1)
  lat_code=$?
  if [ $lat_code -eq 0 ] || [ $lat_code -eq 2 ]; then
    record "latency" PASS "a 10ms profile was applied and read back" "$lat_out"
  else
    record "latency" FAIL "the 10ms profile could not be applied" "$lat_out"
  fi
else
  record "latency" SKIPPED "needs sudo; re-run with --sudo to include it" \
    "dnctl and pfctl require privilege. This path has never been run on a Mac and is the one most likely to be wrong."
fi

# ---- Summary -----------------------------------------------------------------
echo
echo "  $passed passed, $failed failed, $skipped skipped"
echo
echo "Written to $OUT/ (run $RUN_AT):"
ls -1 "$OUT" | grep -- "$RUN_AT" | sed 's/^/  /'
echo
echo "Send it back with:"
echo "  git add $OUT && git commit -m 'macOS verification: run $RUN_AT' && git push"
echo
[ "$failed" -eq 0 ]
