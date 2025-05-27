install-lint-tools:
	cargo +stable install --locked cargo-spellcheck
	cargo +stable install --locked taplo-cli
	cargo +stable install --locked leptosfmt
	cargo +stable install --locked cargo-deny
	
install-lint-tools-ci:
	wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
	tar xzf cargo-binstall-x86_64-unknown-linux-musl.tgz
	cp cargo-binstall ~/.cargo/bin/cargo-binstall
	cargo +stable binstall --no-confirm cargo-spellcheck taplo-cli cargo-deny leptosfmt

lint-all: deny spellcheck fmt-lints cargo-clippy license

fmt:
	cargo fmt --all
	taplo fmt
	leptosfmt .
	corepack enable && yarn && yarn md-fmt

fmt-lints: cargo-fmt taplo md-lint leptos-fmt

md-lint:
	corepack enable && yarn && yarn md-check

cargo-fmt:
	cargo fmt --all --check

leptos-fmt:
	leptosfmt --check .

cargo-clippy:
	cargo clippy --workspace --all-features --all-targets --quiet --no-deps -- --deny warnings

taplo:
	taplo fmt --check
	taplo lint

deny:
	cargo deny check || (echo "See deny.toml"; false)

spellcheck:
	cargo spellcheck --code 1 || (echo "See .config/spellcheck.toml"; false)

# Checks if all headers are present and adds if not
license:
	./scripts/add_license.sh
