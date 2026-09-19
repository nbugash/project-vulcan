#!/usr/bin/env bash
#
# Two operations against this repository's GitHub Actions setup.
#
#   ./tools/ci.sh publish    make the repository public, so Actions minutes on
#                            standard runners stop being billed
#   ./tools/ci.sh run        trigger the gates workflow and watch the macOS job
#
# `publish` is irreversible in the way that matters: once a repository has been
# public, anything in it may have been fetched, cached or indexed by third
# parties, and making it private again does not retract that. It therefore
# refuses to act without --confirm, and refuses outright on anything that looks
# like a secret.

set -euo pipefail
cd "$(dirname "$0")/.."

WORKFLOW="gates.yml"
MACOS_JOB="budgets-authoritative"
# This repository is personal work; the machine's active gh account is the
# company one, and publishing under it would be the wrong account entirely.
EXPECTED_ACCOUNT="${VULCAN_GH_ACCOUNT:-nbugash}"

die() { printf '\nerror: %s\n' "$*" >&2; exit 1; }
note() { printf '  %s\n' "$*"; }

require_gh() {
  command -v gh >/dev/null 2>&1 || die "gh is not installed"
  gh auth status >/dev/null 2>&1 || die "gh is not authenticated; run: gh auth login"
}

active_account() {
  gh auth status 2>&1 | awk '/Logged in to/ {acct=$NF; sub(/\)$/,"",acct); name=$7}
                              /Active account: true/ {print name; exit}'
}

require_account() {
  local active
  active=$(gh auth status 2>&1 | grep -B2 "Active account: true" | grep -oE "account [A-Za-z0-9-]+" | awk '{print $2}' | head -1)
  [ -n "$active" ] || die "could not determine the active gh account"
  if [ "$active" != "$EXPECTED_ACCOUNT" ]; then
    die "active gh account is '$active', expected '$EXPECTED_ACCOUNT'.
       Switch with:  gh auth switch --user $EXPECTED_ACCOUNT
       Or override:  VULCAN_GH_ACCOUNT=$active $0 $*"
  fi
  printf 'account: %s\n' "$active"
}

require_repo() {
  git rev-parse --git-dir >/dev/null 2>&1 \
    || die "not a git repository. Initialise and push it before using this script:
       git init && git add -A && git commit -m 'Initial commit'
       gh repo create <name> --private --source=. --push"
  git remote get-url origin >/dev/null 2>&1 \
    || die "no 'origin' remote. Create the GitHub repository first:
       gh repo create <name> --private --source=. --push"
}

# Anything matching here stops a publish. Better a false positive that costs a
# minute than a credential that cannot be unpublished.
scan_for_secrets() {
  local hits
  hits=$(git grep -In -iE \
    'AKIA[0-9A-Z]{16}|BEGIN (RSA|EC|OPENSSH|PRIVATE) KEY|gh[pousr]_[A-Za-z0-9]{20,}|xox[baprs]-|-----BEGIN CERTIFICATE|password[[:space:]]*=[[:space:]]*["'"'"'][^"'"'"']{6,}' \
    -- . ':(exclude)tools/ci.sh' 2>/dev/null || true)
  if [ -n "$hits" ]; then
    printf '\n%s\n' "$hits" >&2
    die "the above look like credentials. Publishing would expose them permanently."
  fi
  note "secret scan: clean"
}

publish() {
  require_gh
  require_account
  require_repo

  local slug visibility
  slug=$(gh repo view --json nameWithOwner -q .nameWithOwner)
  visibility=$(gh repo view --json visibility -q .visibility)

  printf 'repository: %s\nvisibility: %s\n\n' "$slug" "$visibility"
  if [ "$visibility" = "PUBLIC" ]; then
    note "already public; nothing to do"
    return 0
  fi

  printf 'Making this public would expose:\n'
  note "$(git rev-list --count HEAD) commits of history, including anything ever committed"
  note "$(git ls-files | wc -l | tr -d ' ') tracked files"
  note "every branch and tag pushed to the remote"
  printf '\nIn exchange, Actions minutes on standard runners (including macos-26)\nare not billed for public repositories. Larger runners stay billed.\n'
  scan_for_secrets

  if [ "${1:-}" != "--confirm" ]; then
    printf '\nDRY RUN. Nothing changed. Re-run to apply:\n  %s publish --confirm\n' "$0"
    return 0
  fi

  gh repo edit "$slug" --visibility public --accept-visibility-change-consequences
  printf '\n%s is now public: %s\n' "$slug" "$(gh repo view --json url -q .url)"
}

run_ci() {
  require_gh
  require_repo

  local branch
  branch=$(git rev-parse --abbrev-ref HEAD)
  printf 'workflow: %s\nbranch:   %s\n\n' "$WORKFLOW" "$branch"

  if [ -n "$(git status --porcelain)" ]; then
    note "working tree is dirty; CI runs what is pushed, not what is on disk"
  fi

  gh workflow run "$WORKFLOW" --ref "$branch"

  # `gh workflow run` returns before the run is queued, so wait for an id
  # rather than racing it.
  local id=""
  for _ in $(seq 1 30); do
    sleep 2
    id=$(gh run list --workflow "$WORKFLOW" --branch "$branch" --limit 1 --json databaseId -q '.[0].databaseId' 2>/dev/null || true)
    [ -n "$id" ] && break
  done
  [ -n "$id" ] || die "the run did not appear; check: gh run list --workflow $WORKFLOW"

  printf 'run: %s\n\n' "$id"
  gh run watch "$id" --exit-status || true

  printf '\n--- %s ---\n' "$MACOS_JOB"
  # The machine the numbers came from, which is the point of running this.
  gh run view "$id" --log --job "$(gh run view "$id" --json jobs -q \
    ".jobs[] | select(.name==\"$MACOS_JOB\") | .databaseId")" 2>/dev/null \
    | sed -n '/Observed machine/,/Gate 5/p' || note "could not read the job log; open: $(gh run view "$id" --json url -q .url)"

  gh run view "$id" --json conclusion,url -q '"\nconclusion: \(.conclusion)\nurl: \(.url)"'
}

case "${1:-}" in
  publish) shift; publish "$@" ;;
  run)     shift; run_ci "$@" ;;
  *) printf 'usage: %s {publish [--confirm]|run}\n' "$0" >&2; exit 2 ;;
esac
