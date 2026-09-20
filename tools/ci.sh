#!/usr/bin/env bash
#
# Two operations against this repository's GitHub Actions setup.
#
#   ./tools/ci.sh publish    make the repository public, so Actions minutes on
#                            standard runners stop being billed
#   ./tools/ci.sh run        trigger the gates workflow and watch the macOS job
#   ./tools/ci.sh cycle      make public, run CI, restore the previous visibility
#   ./tools/ci.sh restore    put visibility back, for when a cycle was killed
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

  # A dispatch that does not happen must stop here. Falling through would find
  # the newest run for the branch, which is some earlier push's, and report its
  # verdict as though it were this one's.
  local before
  before=$(gh run list --workflow "$WORKFLOW" --branch "$branch" --limit 1 \
             --json databaseId -q '.[0].databaseId' 2>/dev/null || true)

  if ! gh workflow run "$WORKFLOW" --ref "$branch"; then
    die "could not dispatch $WORKFLOW.
       workflow_dispatch is only offered when the workflow file exists on the
       repository's DEFAULT branch. This repository's default is '$(gh repo view --json defaultBranchRef -q .defaultBranchRef.name)',
       and the workflow is on '$branch'. Merge it to the default branch, or
       change the default, or let a push trigger the run instead."
  fi

  # `gh workflow run` returns before the run is queued, so wait for an id
  # rather than racing it.
  local id=""
  for _ in $(seq 1 30); do
    sleep 2
    id=$(gh run list --workflow "$WORKFLOW" --branch "$branch" --limit 1 --json databaseId -q '.[0].databaseId' 2>/dev/null || true)
    # Only a run that did not exist before the dispatch is this one's.
    [ -n "$id" ] && [ "$id" != "$before" ] && break
    id=""
  done
  [ -n "$id" ] || die "the run did not appear; check: gh run list --workflow $WORKFLOW"

  printf 'run: %s\n\n' "$id"
  gh run watch "$id" --exit-status || true

  printf '\n--- %s ---\n' "$MACOS_JOB"
  # The machine the numbers came from, which is the point of running this.
  gh run view "$id" --log --job "$(gh run view "$id" --json jobs -q \
    ".jobs[] | select(.name==\"$MACOS_JOB\") | .databaseId")" 2>/dev/null \
    | sed -n '/Observed machine/,/Gate 5/p' || note "could not read the job log; open: $(gh run view "$id" --json url -q .url)"

  local conclusion
  conclusion=$(gh run view "$id" --json conclusion -q .conclusion)
  printf "\nconclusion: %s\nurl: %s\n" "$conclusion" \
    "$(gh run view "$id" --json url -q .url)"

  # The caller decides what to do about a failure, but it must be told there was
  # one; reporting 0 for a failed run is worse than not running at all.
  [ "$conclusion" = "success" ]
}

# Where the intended visibility is recorded before anything changes. Inside
# .git/ so it is never committed, and on disk rather than in a variable so that
# a process killed outright still leaves evidence of what it owes.
STAMP=".git/vulcan-visibility-restore"

visibility_of() {
  gh repo view "$1" --json visibility -q .visibility
}

# The finally block. Idempotent, retried, and loud when it cannot finish: a
# repository left public by accident is the failure this whole command exists to
# prevent, so it must never be reported quietly.
restore_visibility() {
  local code=$?
  trap - EXIT INT TERM HUP

  # Nothing here may abort the handler. `set -e` applies inside a trap too, so a
  # single failing command — `kill` on an already-dead process, say — would end
  # the restore before it started. That exact bug left the repository public
  # once; every step below is therefore unconditional.
  set +e

  # Stop watching, if it is still going. The run continues on GitHub either way.
  [ -n "${ci_pid:-}" ] && kill "$ci_pid" 2>/dev/null

  [ -f "$STAMP" ] || exit "$code"
  local slug want
  slug=$(cut -d" " -f1 "$STAMP")
  want=$(cut -d" " -f2 "$STAMP")

  printf "\n--- restoring %s to %s ---\n" "$slug" "$want"

  local attempt current
  for attempt in 1 2 3 4 5; do
    current=$(visibility_of "$slug" 2>/dev/null || echo UNKNOWN)
    if [ "$current" = "$want" ]; then
      rm -f "$STAMP"
      printf "visibility restored to %s\n" "$want"
      exit "$code"
    fi
    [ "$attempt" -gt 1 ] && sleep $(( attempt * 3 ))
    printf "  attempt %d: setting %s ... " "$attempt" "$want"
    if gh repo edit "$slug" --visibility "$(printf %s "$want" | tr "[:upper:]" "[:lower:]")" \
         --accept-visibility-change-consequences >/dev/null 2>&1; then
      printf "ok\n"
    else
      printf "failed\n"
    fi
  done

  # Out of attempts. Say so in terms that cannot be skimmed past.
  current=$(visibility_of "$slug" 2>/dev/null || echo UNKNOWN)
  cat >&2 <<WARNING

!!  COULD NOT RESTORE VISIBILITY
!!
!!  $slug is $current and should be $want.
!!  The repository may still be public. Fix it now, by either:
!!
!!      ./tools/ci.sh restore
!!      gh repo edit $slug --visibility $(printf %s "$want" | tr "[:upper:]" "[:lower:]") --accept-visibility-change-consequences
!!
!!  The note at $STAMP is left in place so a later run retries.

WARNING
  exit 1
}

# Make public, run CI, put it back. The restore runs whether the workflow passes,
# fails, or the command is interrupted.
cycle() {
  require_gh
  require_account
  require_repo

  local slug original
  slug=$(gh repo view --json nameWithOwner -q .nameWithOwner)
  original=$(visibility_of "$slug")

  printf "repository: %s\nvisibility: %s\n\n" "$slug" "$original"

  if [ "$original" = "PUBLIC" ]; then
    note "already public; running CI without changing anything"
    run_ci
    return
  fi

  scan_for_secrets
  printf "\nThis makes %s public for as long as the run takes, then puts it\n" "$slug"
  printf "back to %s. While public it can be read, cloned, forked and indexed\n" "$original"
  printf "by anyone, and that cannot be undone by making it private again.\n" 

  if [ "${1:-}" != "--confirm" ]; then
    printf "\nDRY RUN. Nothing changed. Re-run to apply:\n  %s cycle --confirm\n" "$0"
    return 0
  fi

  # Record the debt before incurring it, so even SIGKILL leaves a trail.
  printf "%s %s %s\n" "$slug" "$original" "$(date -Is)" > "$STAMP"

  # try / finally. EXIT covers success and any `exit`; the signal traps exist so
  # that an interrupt reaches EXIT instead of killing the shell outright.
  trap restore_visibility EXIT
  trap "exit 130" INT
  trap "exit 143" TERM
  trap "exit 129" HUP

  printf "\n--- making public ---\n"
  gh repo edit "$slug" --visibility public --accept-visibility-change-consequences
  printf "public: %s\n" "$(visibility_of "$slug")"

  printf "\n--- running CI ---\n"
  # Run it in the background and wait, rather than in the foreground.
  #
  # bash defers a trap until the current foreground command returns, so an
  # interrupt during `gh run watch` — which blocks for as long as the workflow
  # takes — would not reach the restore until the watch finished on its own.
  # `wait` is interruptible, so the trap fires immediately and the repository
  # goes back to private while the run is still going.
  set +e
  run_ci &
  local ci_pid=$!
  wait "$ci_pid"
  local ci_code=$?
  set -e
  printf "\nCI finished with code %d\n" "$ci_code"

  exit "$ci_code"
}

# For a cycle that was killed before it could put things back.
restore() {
  require_gh
  if [ ! -f "$STAMP" ]; then
    note "no interrupted cycle recorded; nothing to restore"
    return 0
  fi
  trap restore_visibility EXIT
  exit 0
}

case "${1:-}" in
  publish) shift; publish "$@" ;;
  run)     shift; run_ci "$@" ;;
  cycle)   shift; cycle "$@" ;;
  restore) shift; restore "$@" ;;
  *) printf 'usage: %s {publish [--confirm]|run|cycle [--confirm]|restore}\n' "$0" >&2; exit 2 ;;
esac
