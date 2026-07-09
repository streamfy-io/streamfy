install-fmt:
	rustup component add rustfmt --toolchain $(RUSTV)

check-fmt:
	cargo +$(RUSTV) fmt -- --check

check-docs:
	cargo doc --no-deps

check_version:
	make check_version -C k8-util/helm

install-clippy:
	rustup component add clippy --toolchain $(RUSTV)

# Use check first to leverage sccache, the clippy piggybacks
check-clippy: install-clippy install_rustup_target
	cargo +$(RUSTV) check --all --all-features --tests $(VERBOSE_FLAG) $(TARGET_FLAG)
	cargo +$(RUSTV) clippy --all --all-features --tests $(VERBOSE_FLAG) -- -D warnings -A clippy::upper_case_acronyms $(TARGET_FLAG)

install-udeps:
	cargo install cargo-udeps --locked

check-udeps: install-udeps
	cargo +nightly udeps --all-targets

install-deny:
	cargo install --locked cargo-deny

check-crate-audit: install-deny
	cargo deny check

build_smartmodules:
	make -C smartmodule/examples build

# TLS unit tests in streamfy-future read certs/test-certs/* relative to that crate.
# Those files are gitignored and must be generated with openssl before testing.
.PHONY: generate-streamfy-future-certs
generate-streamfy-future-certs:
	$(MAKE) -C crates/streamfy-future certs

run-all-unit-test: install_rustup_target generate-streamfy-future-certs
	cargo test --lib --all-features $(BUILD_FLAGS)
	cargo test -p streamfy-smartmodule $(BUILD_FLAGS)
	cargo test -p streamfy-storage $(BUILD_FLAGS)
	cargo test -p streamfy-channel-cli $(BUILD_FLAGS)
	cargo test -p streamfy-connector-derive $(BUILD_FLAGS)
	cargo test -p streamfy-connector-common --all-features $(BUILD_FLAGS)
	cargo test -p streamfy-connector-package $(BUILD_FLAGS)
	cargo test -p streamfy-controlplane-metadata --features=smartmodule $(BUILD_FLAGS)
	make test-all -C crates/streamfy-protocol

run-integration-test: build_smartmodules install_rustup_target
	cargo test  --lib --all-features $(BUILD_FLAGS) -p streamfy-spu -- --ignored --test-threads=1
	cargo test  --lib --all-features $(BUILD_FLAGS) -p streamfy-socket -- --ignored --test-threads=1
	cargo test  --lib --all-features $(BUILD_FLAGS) -p streamfy-service -- --ignored --test-threads=1
	cargo test -p streamfy-smartengine -- --ignored --test-threads=1

run-smartmodule-test:	build_smartmodules
	cargo test  -p streamfy-smartengine -- --ignored --nocapture

run-k8-test:	install_rustup_target k8-setup build_k8_image
	cargo test --lib  -p streamfy-sc  -- --ignored --test-threads=1


run-all-doc-test: install_rustup_target
	cargo test --all-features --doc  $(BUILD_FLAGS)

run-client-doc-test: install_rustup_target
	cargo test --all-features --doc -p streamfy-cli $(BUILD_FLAGS)
	cargo test --all-features --doc -p streamfy-cluster $(BUILD_FLAGS)
	cargo test --all-features --doc -p streamfy $(BUILD_FLAGS)


streamfy_run_bin: install_rustup_target
	cargo zigbuild --bin streamfy-run -p streamfy-run $(RELEASE_FLAG) --target $(TARGET) $(DEBUG_SMARTMODULE_FLAG)
