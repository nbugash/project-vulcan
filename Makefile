# Vulcan — task runner.
#
# `make` on its own lists what is here. Every target is a thin wrapper around a
# command you could type yourself; nothing is hidden behind make magic, because
# a task runner that obscures the command it runs makes failures harder to read,
# not easier.
#
# Gate exit codes are three, not two: 0 judged and passed, 2 judged and failed,
# 1 could not judge. make stops on any non-zero, which is the intent — a gate
# that could not run is not a pass.

SHELL := /bin/bash
.DEFAULT_GOAL := help

# The sequencing extension's tests need pytest, which this machine's Python
# refuses to install into system site-packages (PEP 668). A venv under target/
# keeps it out of the way and out of git.
VENV := target/py
PY := $(VENV)/bin/python
FEATURE_MAP := .specify/extensions/featuremap/scripts/python/feature_map.py
PINNED := ./tools/gate-fidelity/run-in-pinned-env.sh

.PHONY: help
help: ## List the targets
	@echo "Vulcan"
	@echo
	@grep -hE '^[a-zA-Z0-9_.-]+:.*?## ' $(MAKEFILE_LIST) \
		| awk -F':.*?## ' '{printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}'
	@echo
	@echo "  Gate exit codes: 0 passed, 2 failed, 1 could not judge."

# ---- Development -------------------------------------------------------------

.PHONY: setup
setup: ## Check this machine has what Vulcan needs
	./tools/setup.sh

.PHONY: setup-install
setup-install: ## The same, and set up the optional Python tooling
	./tools/setup.sh --install

.PHONY: build
build: ## Compile the workspace
	cargo build --workspace

.PHONY: test
test: ## Run the Rust test suite
	cargo test --workspace

.PHONY: fmt
fmt: ## Format the Rust sources
	cargo fmt --all

.PHONY: run
run: ## Open the shell in a window
	cargo run -p shell-preview

.PHONY: smoke
smoke: ## Resolve the layout with no display at all
	cargo run -q -p shell-preview -- --smoke

# ---- Gates -------------------------------------------------------------------

.PHONY: gates
gates: test boundary lint triage sequence prerequisites ## Everything that runs without the pinned environment
	@echo
	@echo "All gates that can run outside the pinned environment passed."
	@echo "Still to run: 'make compare' (needs the pinned session) and"
	@echo "'make budgets' (needs a machine whose constraints it can enforce)."

.PHONY: boundary
boundary: ## Gate 1 — architectural boundaries
	cargo run -q -p gate-boundary

.PHONY: tokens
tokens: ## Gate 8 — extract design values from the prototype
	cargo run -q -p gate-fidelity -- extract

.PHONY: lint
lint: ## Gate 8 — reject design values the prototype does not define
	cargo run -q -p gate-fidelity -- lint

.PHONY: triage
triage: ## Gate 8 — every discrepancy is decided and linked
	cargo run -q -p gate-fidelity -- discrepancies

.PHONY: compare
compare: ## Gate 8 — exact pixel comparison, inside the pinned session
	$(PINNED) compare

.PHONY: reference
reference: ## Gate 8 — re-approve the baseline the comparison judges against
	@echo "This replaces the approved reference. Only do it when the interface"
	@echo "was meant to change, and look at the result before committing it."
	$(PINNED) capture-reference

.PHONY: budgets
budgets: ## Gate 5 — resource budgets (advisory, on Linux)
	cargo run -q -p gate-budget -- --runner linux-cgroup

.PHONY: budgets-mac
budgets-mac: ## Gate 5 — resource budgets (authoritative, on Apple Silicon)
	cargo run -q -p gate-budget -- --runner apple-silicon

.PHONY: sequence
sequence: $(PY) ## Gate 9 — the feature map is consistent, and its own tests
	python3 $(FEATURE_MAP) verify
	python3 $(FEATURE_MAP) resolve
	$(PY) -m pytest .specify/extensions/featuremap/tests/ -q

.PHONY: prerequisites
prerequisites: ## Every gate refuses rather than passes without its prerequisites
	./tools/check-prerequisites.sh

# ---- Assets and captures -----------------------------------------------------

.PHONY: fonts
fonts: ## Rebuild assets/fonts from the woff2 the prototype vendors
	./tools/build-fonts.sh

.PHONY: screenshots
screenshots: ## Capture every notable state for review (gitignored)
	./tools/screenshots.sh

# ---- CI and publishing -------------------------------------------------------

.PHONY: ci
ci: ## Trigger the gates workflow and watch the macOS job
	./tools/ci.sh run

.PHONY: publish
publish: ## Make the repository public — DRY RUN, prints what it would do
	./tools/ci.sh publish

.PHONY: publish-confirm
publish-confirm: ## Make the repository public, for real. Irreversible in effect.
	./tools/ci.sh publish --confirm

.PHONY: cycle
cycle: ## Public -> run CI -> restore visibility. DRY RUN.
	./tools/ci.sh cycle

.PHONY: cycle-confirm
cycle-confirm: ## Public -> run CI -> restore visibility, for real
	./tools/ci.sh cycle --confirm

.PHONY: restore
restore: ## Put visibility back after a cycle that was killed
	./tools/ci.sh restore

.PHONY: visibility-token
visibility-token: ## Store the PAT the CI restore job needs
	./tools/setup-visibility-token.sh

.PHONY: check-visibility-token
check-visibility-token: ## Check that PAT can do what the restore job needs
	./tools/check-visibility-token.sh

# ---- macOS -------------------------------------------------------------------

.PHONY: macos-verify
macos-verify: ## Run on a Mac: the checks only a Mac can answer
	./tools/macos-verify.sh

.PHONY: macos-verify-sudo
macos-verify-sudo: ## The same, including the latency profiles, which need sudo
	./tools/macos-verify.sh --sudo

# ---- Housekeeping ------------------------------------------------------------

$(PY):
	python3 -m venv $(VENV)
	$(VENV)/bin/pip install --quiet pytest

.PHONY: clean
clean: ## Remove build output and generated captures
	cargo clean
	rm -rf reports/screenshots reports/budgets tools/gate-fidelity/out
