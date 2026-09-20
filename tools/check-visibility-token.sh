#!/usr/bin/env bash
# Checks that the token behind VISIBILITY_TOKEN can do the one thing the CI
# restore job needs, and no more than it should.
#
# A stored Actions secret cannot be read back — not by the API, not by gh, not
# by this script. So the token is asked for again here, used directly against
# the API, and never written anywhere: no file, no shell history, no log.
set -euo pipefail
cd "$(dirname "$0")/.."

REPO="${1:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"
pass=0; fail=0; warn=0

ok()   { printf '  ok    %s\n' "$*"; pass=$((pass+1)); }
bad()  { printf '  FAIL  %s\n' "$*"; fail=$((fail+1)); }
note() { printf '  note  %s\n' "$*"; warn=$((warn+1)); }

if [ -n "${VISIBILITY_TOKEN:-}" ]; then
  token="$VISIBILITY_TOKEN"
else
  read -rsp "Paste the token to check (input hidden, not stored): " token
  echo
fi
[ -n "$token" ] || { echo "no token given" >&2; exit 1; }

echo
echo "Checking the token against $REPO"
echo

hdr=$(mktemp); trap 'rm -f "$hdr"' EXIT

# --- identity ---------------------------------------------------------------
who=$(GH_TOKEN="$token" gh api user --jq .login 2>/dev/null || echo "")
owner="${REPO%%/*}"
if [ -z "$who" ]; then
  bad "the token is not valid, or cannot read /user"
elif [ "$who" = "$owner" ]; then
  ok "belongs to $who, which owns $REPO"
else
  bad "belongs to $who, but $REPO is owned by $owner"
fi

# --- expiry and classic scopes ----------------------------------------------
GH_TOKEN="$token" gh api repos/"$REPO" --include >"$hdr" 2>/dev/null || true
expiry=$(grep -i '^github-authentication-token-expiration:' "$hdr" | cut -d' ' -f2- | tr -d '\r' || true)
scopes=$(grep -i '^x-oauth-scopes:' "$hdr" | cut -d' ' -f2- | tr -d '\r' || true)

if [ -n "$expiry" ]; then
  ok "expires $expiry"
else
  note "no expiry reported; a token that never expires is a standing risk"
fi

if [ -n "$scopes" ]; then
  # A classic PAT. `repo` is coarse: it grants far more than visibility.
  note "classic token, scopes: $scopes"
  note "a fine-grained token limited to this repo and Administration would be tighter"
else
  ok "fine-grained token (no coarse OAuth scopes)"
fi

# --- read access -------------------------------------------------------------
current=$(GH_TOKEN="$token" gh api repos/"$REPO" --jq .visibility 2>/dev/null || echo "")
if [ -n "$current" ]; then
  ok "can read $REPO (currently $current)"
else
  bad "cannot read $REPO — repository access is not granted to this token"
fi

# --- the permission that actually matters ------------------------------------
# Administration: write, exercised by setting visibility to what it already is.
# A no-op that still goes through the same authorisation path as the real
# change, so it proves the permission without altering anything.
if [ -n "$current" ]; then
  code=$(GH_TOKEN="$token" gh api -X PATCH "repos/$REPO" \
           -f visibility="$(printf %s "$current" | tr '[:upper:]' '[:lower:]')" \
           --silent --include 2>/dev/null | head -1 | awk '{print $2}' || echo "")
  after=$(GH_TOKEN="$token" gh api repos/"$REPO" --jq .visibility 2>/dev/null || echo "")
  if [ "$code" = "200" ]; then
    ok "can change visibility (Administration: write) — no-op PATCH accepted"
  else
    bad "cannot change visibility; the CI restore job would fail and leave the repo public"
    note "grant Administration: read and write, for this repository only"
  fi
  [ "$after" = "$current" ] && ok "visibility unchanged by the check ($after)" \
                            || bad "visibility changed during the check: $current -> $after"
fi

unset token
echo
printf '  %d passed, %d failed, %d to consider\n' "$pass" "$fail" "$warn"
[ "$fail" -eq 0 ]
