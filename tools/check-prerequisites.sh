#!/usr/bin/env bash
# T077 / quickstart: every gate must exit 1 — never 0 — when a prerequisite is
# missing. A gate that reports a pass for a check it did not run is the one
# failure that makes the whole suite worthless.
set -u
cd "$(dirname "$0")/.."

failures=0

expect_refusal() {
  local label="$1"; shift
  set +e
  local output
  output=$("$@" 2>&1)
  local code=$?
  set -e
  if [ "$code" -eq 1 ]; then
    printf '  ok    %-46s exit 1\n' "$label"
  else
    printf '  FAIL  %-46s exit %s (want 1)\n' "$label" "$code"
    printf '        %s\n' "$(printf '%s' "$output" | head -1)"
    failures=$((failures + 1))
  fi
}

echo "Prerequisite refusals:"

expect_refusal "gate 1, no workspace at the given path" \
  cargo run --quiet -p gate-boundary -- --manifest-path /nonexistent-workspace

expect_refusal "gate 8 extract, no prototype" \
  cargo run --quiet -p gate-fidelity -- extract --prototype /nonexistent-mockups

expect_refusal "gate 8 lint, no prototype" \
  cargo run --quiet -p gate-fidelity -- lint --prototype /nonexistent-mockups

expect_refusal "gate 8 discrepancies, no feature map" \
  cargo run --quiet -p gate-fidelity -- discrepancies --map /nonexistent-map.md

expect_refusal "gate 8 compare, outside the pinned environment" \
  cargo run --quiet -p gate-fidelity -- compare

expect_refusal "gate 8 capture-reference, outside the pinned environment" \
  cargo run --quiet -p gate-fidelity -- capture-reference

expect_refusal "gate 5, constraints not enforceable" \
  cargo run --quiet -p gate-budget -- --runner linux-cgroup

# Gate 9 is deliberately different, and the difference is a project policy
# rather than a defect in the extension. `feature_map.py` exits 0 with SKIPPED
# when no map is present, which is right for spec-kit in general: a project that
# does not sequence features should not be blocked by a gate about sequencing.
#
# Principle IX makes the map mandatory *here*, so "not in use" is not a state
# this repository may be in. The extension cannot know that, so the repository
# asserts it.
echo
echo "Project policy:"
if [ -f specs/features-map.md ]; then
  printf '  ok    %-46s present\n' "feature map (Principle IX requires it)"
else
  printf '  FAIL  %-46s absent; gate 9 would SKIP and exit 0\n' "feature map (Principle IX requires it)"
  failures=$((failures + 1))
fi

if [ "$failures" -ne 0 ]; then
  echo "$failures gate(s) did not refuse correctly" >&2
  exit 1
fi
echo "all gates refuse rather than pass when a prerequisite is missing"
