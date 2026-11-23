CARGO_HOME ?= $(HOME)/.cargo
HEISENBERG_ARGUMENTS := --cargo-home "$(CARGO_HOME)" --sysroot "$(shell rustc --print sysroot)" --configuration x86_64-unknown-linux-gnu --workspace "$(CURDIR)"
RUST_SOURCES := $(wildcard */src/*.rs */tests/*.rs)
WORKSPACE_MANIFESTS := $(wildcard Cargo.lock Cargo.toml */Cargo.toml)

.DELETE_ON_ERROR:

target/llvm-cov.json: $(RUST_SOURCES) $(WORKSPACE_MANIFESTS) .config/nextest.toml rust-toolchain.toml
	cargo llvm-cov clean --workspace
	cargo llvm-cov nextest --no-report --workspace --lib --bins --tests --examples
	cargo llvm-cov report --json --include-build-script --output-path $@

.PHONY: clippy
clippy:
	cargo clippy --workspace --tests

.PHONY: coverage
coverage: target/llvm-cov.json
	cargo fetch
	heisenberg coverage-check $(HEISENBERG_ARGUMENTS) --coverage-export "$(CURDIR)/$<"

.PHONY: fmt
fmt:
	cargo fmt

.PHONY: heisenberg
heisenberg:
	cargo fetch
	heisenberg check $(HEISENBERG_ARGUMENTS)

.PHONY: rust-test
rust-test:
	cargo nextest run --workspace
