default: build

SMDK_BIN ?= smdk
SMDK_OPT ?=

# filter_with_param_v1 uses the pre-init #[smartmodule(filter, params)] API which
# current streamfy-smartmodule-derive rejects. Keep sources for reference; skip build.
SUBDIRS := $(filter-out target/. filter_with_param_v1/., $(wildcard */.))

.PHONY: build $(SUBDIRS)

setup:
	rustup target add wasm32-wasip1
	@echo SMDK_BIN ${SMDK_BIN}

dirs:
	echo ${SUBDIRS}

build: setup $(SUBDIRS)

$(SUBDIRS):
	@echo Building smartmodule example $@
	cd $@ && ${SMDK_BIN} build ${SMDK_OPT}

build-raw: setup
	cargo build --release

clean:
	cargo clean
