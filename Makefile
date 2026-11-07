CARGO_HOME ?= $(HOME)/.cargo

# -----------------------------------------------------------------------------
# Real targets
# -----------------------------------------------------------------------------

node_modules: package-lock.json
	npm ci
	touch node_modules

package-lock.json: package.json
	npm install --package-lock-only

# -----------------------------------------------------------------------------
# Phony targets
# -----------------------------------------------------------------------------

.PHONY: clippy
clippy:
	cargo clippy --workspace --tests

.PHONY: coverage
coverage: node_modules
	cargo llvm-cov clean --workspace
	cargo llvm-cov --workspace --no-report
	cargo llvm-cov report --json --output-path target/llvm-cov.json
	npx rust-coverage-check target/llvm-cov.json \
		--workspace-root $(CURDIR) \
		--gated ephemeral-postgres=100 \
		--gated ephemeral-postgres-run=100

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: heisenberg
heisenberg:
	cargo fetch
	heisenberg check \
		--cargo-home "$(CARGO_HOME)" \
		--sysroot "$(shell rustc --print sysroot)" \
		--configuration x86_64-unknown-linux-gnu \
		--workspace "$(CURDIR)"

.PHONY: rust-test
rust-test:
	cargo test --workspace
