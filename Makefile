.PHONY: fmt lint typecheck test check-all

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

typecheck:
	cargo check --all-targets --all-features

test:
	cargo test --all-features

check-all: lint typecheck test
