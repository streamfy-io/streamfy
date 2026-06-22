# Build targets
build-cli: install_rustup_target
	$(CARGO_BUILDER) build --bin streamfy -p streamfy-cli $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG) $(SMARTENGINE_FLAG)

build-smdk: install_rustup_target
	$(CARGO_BUILDER) build --bin smdk -p smartmodule-development-kit $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG) $(SMARTENGINE_FLAG)

build-cdk: install_rustup_target
	$(CARGO_BUILDER) build --bin cdk -p cdk $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG) $(SMARTENGINE_FLAG)

build-benchmark: install_rustup_target
	$(CARGO_BUILDER) build --bin streamfy-benchmark -p streamfy-benchmark $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG) $(SMARTENGINE_FLAG)

build-fvm: install_rustup_target
	$(CARGO_BUILDER) build --bin fvm -p streamfy-version-manager $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG) $(SMARTENGINE_FLAG)

build-cli-minimal: install_rustup_target
	# https://github.com/streamfy-io/streamfy/issues/1255
	cargo build --bin streamfy -p streamfy-cli $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG) \
	    --no-default-features --features consumer,producer-file-io

# note: careful that the if statement branches are leading spaces, tabs
ifeq ($(TARGET), armv7-unknown-linux-gnueabihf)
  streamfy_run_extra=--no-default-features --features rustls
else
  streamfy_run_extra=
endif
build-cluster: install_rustup_target
	cargo build --bin streamfy-run -p streamfy-run $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG) $(DEBUG_SMARTMODULE_FLAG) $(streamfy_run_extra)

build-run:
	cargo build --bin streamfy-run -p streamfy-run $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG) $(DEBUG_SMARTMODULE_FLAG) $(streamfy_run_extra)

build-test:	install_rustup_target
	cargo build --bin streamfy-test -p streamfy-test $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG)

build-channel: install_rustup_target
	$(CARGO_BUILDER) build --bin streamfy-channel -p streamfy-channel-cli $(RELEASE_FLAG) $(TARGET_FLAG) $(VERBOSE_FLAG)

install_rustup_target:
	./build-scripts/install_target.sh


build_k8_image: K8_CLUSTER?=$(shell ./k8-util/cluster/cluster-type.sh)

# In CI mode, do not build k8 image
ifeq (${CI},true)
build_k8_image:
else ifeq (${IMAGE_VERSION},true)
build_k8_image:
else ifeq (${STREAMFY_MODE},local)
build_k8_image:
else
# When not in CI (i.e. development), build image before testing
build_k8_image: streamfy_image
endif


# Build docker image for Streamfy.
ifndef TARGET
ifeq ($(ARCH),arm64)
streamfy_image: TARGET=aarch64-unknown-linux-musl
else
streamfy_image: TARGET=x86_64-unknown-linux-musl
endif
endif
streamfy_image: streamfy_run_bin
	echo "Building Streamfy $(TARGET) image with tag: $(GIT_COMMIT) k8 type: $(K8_CLUSTER)"
	k8-util/docker/build.sh $(TARGET) $(GIT_COMMIT) "./target/$(TARGET)/$(BUILD_PROFILE)/streamfy-run" $(K8_CLUSTER)
