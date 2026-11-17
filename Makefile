# SPDX-FileCopyrightText: 2026 Adrian Wojak <adrian.wojak@outlook.com>
# SPDX-License-Identifier: GPL-3.0-or-later

.DEFAULT_GOAL := build-debug

.PHONY: build-debug build-release check test format package clean

build-debug:
	cargo build --workspace --locked

build-release:
	cargo build --workspace --release --locked

check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --locked -- -D warnings
	nixfmt --check flake.nix
	statix check .
	deadnix --fail .
	actionlint

test:
	cargo test --workspace --locked

format:
	cargo fmt --all
	nixfmt flake.nix

package: build-release
	mkdir -p output/dist
	@set -eu; \
	version=$$(git describe --tags --long --always --dirty); \
	version=$$(printf '%s' "$$version" | tr '/' '-'); \
	archive="output/dist/amanita-x86_64-linux-$$version.tar.gz"; \
	tar -czf "$$archive" -C output/target/x86_64-unknown-linux-musl/release amanita -C "$$(pwd)" LICENSE; \
	printf '%s\n' "$$archive"

clean:
	rm -rf -- output
