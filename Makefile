install-lint-tools:
	cargo install --locked cargo-spellcheck@0.15.1 # later versions require higher MSRV
	cargo install --locked taplo-cli
	cargo install --locked cargo-deny
	
install-lint-tools-ci:
	wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
	tar xzf cargo-binstall-x86_64-unknown-linux-musl.tgz
	cp cargo-binstall ~/.cargo/bin/cargo-binstall
	cargo binstall --no-confirm cargo-spellcheck@0.15.1 taplo-cli cargo-deny

lint-all: deny spellcheck fmt-lints cargo-clippy

fmt:
	cargo fmt --all
	taplo fmt
	corepack enable && yarn && yarn md-fmt

fmt-lints: cargo-fmt taplo md-lint

md-lint:
	corepack enable && yarn && yarn md-check

cargo-fmt:
	cargo fmt --all --check

cargo-clippy:
	cargo clippy --workspace --all-features --all-targets --quiet --no-deps -- --deny warnings

taplo:
	taplo fmt --check
	taplo lint

deny:
	cargo deny check || (echo "See deny.toml"; false)

spellcheck:
	cargo spellcheck --code 1 || (echo "See .config/spellcheck.toml"; false)
