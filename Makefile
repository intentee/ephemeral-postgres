COVERAGE_PACKAGES := -p ephemeral-postgres

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
	cargo clippy --workspace --tests -- -D warnings

.PHONY: coverage
coverage: node_modules
	cargo llvm-cov clean --workspace
	cargo llvm-cov $(COVERAGE_PACKAGES) --no-report
	cargo llvm-cov report --json --output-path target/llvm-cov.json
	npx rust-coverage-check target/llvm-cov.json \
		--workspace-root $(CURDIR) \
		--gated ephemeral-postgres=100

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: rust-test
rust-test:
	cargo test --workspace
