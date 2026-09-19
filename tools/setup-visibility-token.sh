#!/usr/bin/env bash
# Stores the token the CI restore job uses to make this repository private again.
#
# The workflow's built-in GITHUB_TOKEN cannot change repository visibility at any
# permission level, so that job needs a token of its own. It is held as an
# encrypted Actions secret — never a file in the repository, because anything
# committed stays in the history and this repository is meant to go public.
set -euo pipefail
cd "$(dirname "$0")/.."

SECRET="VISIBILITY_TOKEN"

cat <<'GUIDE'
Create a fine-grained personal access token:

  https://github.com/settings/personal-access-tokens/new

    Resource owner      nbugash
    Repository access   Only select repositories -> project-vulcan
    Permissions         Repository permissions -> Administration: Read and write
    Expiration          the shortest that is workable; 30 days is plenty

  Administration is what allows a visibility change. Do not grant anything else:
  this token only has to flip one switch, and the repository it guards will be
  publicly readable while CI runs.

GUIDE

read -rsp "Paste the token (input hidden): " token
echo
[ -n "$token" ] || { echo "no token given" >&2; exit 1; }

printf '%s' "$token" | gh secret set "$SECRET" --app actions
unset token

echo
gh secret list --app actions | sed 's/^/  /'
echo
echo "Stored as $SECRET. It is write-only from here: GitHub will not show it again,"
echo "and it is masked in workflow logs."
echo
echo "Check it can do what the restore job needs, while you still have it to hand:"
echo "  make check-visibility-token"
 