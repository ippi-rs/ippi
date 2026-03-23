.PHONY: all build build-x86 build-arm64 build-armv7 build-armv6 build-all clean test clippy fmt

FEATURES_FULL = frontend-embedded,p2p-full,kvm
FEATURES_ARM = frontend-embedded,p2p-full

all: build

build:
	cargo build --release --features $(FEATURES_FULL)

build-x86:
	cargo zigbuild --release --target x86_64-unknown-linux-gnu --features $(FEATURES_FULL)

build-arm64:
	cargo zigbuild --release --target aarch64-unknown-linux-gnu --features $(FEATURES_FULL)

build-armv7:
	cargo zigbuild --release --target armv7-unknown-linux-gnueabihf --no-default-features --features $(FEATURES_ARM)

build-armv6:
	cargo zigbuild --release --target arm-unknown-linux-gnueabihf --no-default-features --features $(FEATURES_ARM)

build-all: build-x86 build-arm64 build-armv7 build-armv6

test:
	cargo test --features $(FEATURES_FULL)

clippy:
	cargo clippy --features $(FEATURES_FULL) -- -D warnings

fmt:
	cargo fmt --all -- --check

clean:
	cargo clean
