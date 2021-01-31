DIR_BUILD     := target
DIR_RELEASE   := ${DIR_BUILD}/release
BIN_NAME      := "!set_this!"
.DEFAULT_GOAL := all

ifdef $(INIT_PROJ)
	wget http://launchpadlibrarian.net/486729287/libproj19_7.1.0-1_amd64.deb
	wget http://launchpadlibrarian.net/486729286/proj-data_7.1.0-1_all.deb
	wget http://launchpadlibrarian.net/486729288/proj-bin_7.1.0-1_amd64.deb
	dpkg -i *.deb
	apt-get install libtiff-dev libsqlite3-dev
endif

.PHONY: all
all : build test

.PHONY: init
init :
	rustup toolchain install nightly
	rustup override set nightly
	rustup component add clippy
	rustup component add rustfmt
	cargo install cargo-watch --locked cargo
	cargo install cargo-edit --locked cargo
	cargo install cargo-tarpaulin --locked cargo
	cargo install cargo-audit --locked cargo
	cargo install cargo-outdated --locked cargo
	cargo install cargo-release --locked cargo
	cargo install cargo-udeps --locked cargo
	cargo install cargo-geiger --locked cargo

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
