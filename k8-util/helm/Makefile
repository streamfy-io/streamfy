MAKE_DIR = $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
# Strip whitespace/newlines so helm --app-version is a clean string
VERSION := $(shell tr -d '[:space:]' < $(MAKE_DIR)/../../VERSION)
APP_VERSION = $(VERSION)

SYS_TGZ = pkg_sys/streamfy-chart-sys.tgz
APP_TGZ = pkg_app/streamfy-chart-app.tgz

.PHONY: clean package

# remove packaged charts
# must clean if any of the contents changed due to limitation of "build.rs"
clean:
	rm -rf pkg_sys
	rm -rf pkg_app

# Package a chart directory into a stable tgz name.
# Prefer `helm package` when helm is installed; fall back to tar so
# `include_dir!` embeds (and unit tests) still work without helm.
#
# Important: do not use the package *directory* as the make target —
# an empty dir would look "up to date" and skip packaging.
define package_chart
	set -e; \
	mkdir -p $(1); \
	rm -f $(1)/$(3); \
	if command -v helm >/dev/null 2>&1; then \
		echo "Packaging $(2) with helm (app-version=$(APP_VERSION))"; \
		helm package ./$(2) --app-version "$(APP_VERSION)" -d $(1); \
		mv -f $(1)/$(2)-*.tgz $(1)/$(3); \
	else \
		echo "helm not found; packaging $(2) with tar into $(1)/$(3)"; \
		tar -czf $(1)/$(3) $(2); \
	fi; \
	test -s $(1)/$(3); \
	ls -la $(1)/$(3)
endef

# Chart sources (include templates so chart changes force repackage)
SYS_SRCS := $(wildcard streamfy-sys/*.*) $(wildcard streamfy-sys/templates/*)
APP_SRCS := $(wildcard streamfy-app/*.*) $(wildcard streamfy-app/templates/*)

$(SYS_TGZ): $(SYS_SRCS) ../../VERSION
	$(call package_chart,pkg_sys,streamfy-sys,streamfy-chart-sys.tgz)

$(APP_TGZ): $(APP_SRCS) ../../VERSION
	$(call package_chart,pkg_app,streamfy-app,streamfy-chart-app.tgz)

package: $(SYS_TGZ) $(APP_TGZ)
	@echo "Packaged charts are in pkg_sys and pkg_app"
	@ls -la pkg_sys pkg_app
	@test -s $(SYS_TGZ)
	@test -s $(APP_TGZ)
