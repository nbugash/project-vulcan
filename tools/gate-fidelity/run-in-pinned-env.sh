#!/usr/bin/env bash
# Run a gate-fidelity subcommand inside the pinned rendering environment.
#
# GPUI presents a Vulkan swapchain. On X11 that needs DRI3, which virtual
# framebuffers do not provide, so capture under Xvfb or Xorg's dummy driver
# returns an empty image. Wayland presents through wl_shm instead, and wlroots'
# headless backend supplies the virtual seat GPUI requires, so this runs a
# headless sway session and captures through wlr-screencopy.
#
# The same image runs locally and in the build, so verdicts agree (FR-019).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
IMAGE="${VULCAN_PINNED_IMAGE:-vulcan-pinned-env:3}"
VIEWPORT="${VULCAN_VIEWPORT:-1440x900}"
RUNTIME="${CONTAINER_RUNTIME:-$(command -v podman || command -v docker || true)}"

# Inside the pinned environment already: start a session and run the gate.
if [[ "${VULCAN_PINNED_ENV:-0}" == "1" && "${VULCAN_SESSION:-0}" != "1" ]]; then
  export VULCAN_SESSION=1
  # Wayland requires the runtime directory to be owned by the running user at
  # mode 0700. A path baked into the image cannot satisfy that, because the
  # caller chooses the uid — the image assumed 1000 and CI runs as 1001, so sway
  # could not create its socket and the compositor never started. Deriving the
  # path from the uid under /tmp works for any of them.
  runtime="${XDG_RUNTIME_DIR:-/tmp/vulcan-$(id -u)}"
  mkdir -p "${runtime}"
  chmod 700 "${runtime}"
  export XDG_RUNTIME_DIR="${runtime}"

  config="$(mktemp)"
  {
    echo "output HEADLESS-1 mode ${VIEWPORT}"
    # No decorations: the reference must contain the product's pixels and
    # nothing the compositor drew around them.
    echo "default_border none"
    echo "default_floating_border none"
    echo "exec ${0} __in_session__ $*"
  } > "${config}"

  # Validate before starting. sway shows an error screen for a bad config and
  # carries on, and `grim` then captures that screen: a run once approved a
  # picture of sway's "There are errors in your config file" dialog as the
  # signed-off reference. A compositor that did not start the way we asked must
  # stop the gate, not decorate it.
  if ! sway -c "${config}" -C >/dev/null 2>&1; then
    echo "ERROR: the session config is invalid:" >&2
    sway -c "${config}" -C 2>&1 | grep -i "error on line" >&2
    exit 1
  fi

  exec sway -c "${config}"
fi

# Inside a session: run the gate, then exit the compositor with its code.
if [[ "${1:-}" == "__in_session__" ]]; then
  shift
  # The gate spawns the shell, so the shell has to exist. `cargo run -p
  # gate-fidelity` builds the gate and nothing else, and the capture then failed
  # with "No such file or directory" for a binary no step had produced.
  cargo build --quiet -p shell-preview || exit 1

  set +e
  cargo run --quiet -p gate-fidelity -- "$@"
  code=$?
  set -e
  swaymsg exit >/dev/null 2>&1 || true
  exit "${code}"
fi

# On the host: build the image if needed, then re-enter inside it.
if [[ -z "${RUNTIME}" ]]; then
  echo "ERROR: no container runtime found; the pinned environment is required" >&2
  exit 1
fi

if ! "${RUNTIME}" image inspect "${IMAGE}" >/dev/null 2>&1; then
  "${RUNTIME}" build -f "${REPO_ROOT}/tools/gate-fidelity/pinned-env/Containerfile" -t "${IMAGE}" "${REPO_ROOT}"
fi

exec "${RUNTIME}" run --rm \
  -v "${REPO_ROOT}:/work:z" \
  --user "$(id -u):$(id -g)" \
  -e VULCAN_PINNED_ENV=1 \
  -e HOME=/tmp \
  -e VULCAN_VIEWPORT="${VIEWPORT}" \
  "${IMAGE}" \
  /work/tools/gate-fidelity/run-in-pinned-env.sh "$@"
