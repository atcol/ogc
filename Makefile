DIR_BUILD     := target
DIR_RELEASE   := ${DIR_BUILD}/release
BIN_NAME      := "!set_this!"
.DEFAULT_GOAL := all

.PHONY: all
all : build test

.PHONY: init
init :
	rustup toolchain install nightly
	rustup override set nightly
	rustup component add clippy
	rustup component add rustfmt
	cargo install cargo-watch
	cargo install cargo-edit
	cargo install cargo-tarpaulin
	cargo install cargo-audit
	cargo add proptest
	cargo add tokio

.PHONY: build
build ${DIR_RELEASE}/${BIN_NAME} :
	cargo build --release

.PHONY: test
test :
	cargo test --verbose && \
	  cargo rustdoc

.PHONY: watch
watch :
	cargo-watch -x "test && cargo rustdoc"
