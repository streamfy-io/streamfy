MAKE_DIR = $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
# Strip whitespace/newlines so helm --app-version is a clean semver-ish string
VERSION := $(shell tr -d '[:space:]' < $(MAKE_DIR)/../../VERSION)
APP_VERSION = $(VERSION)

# remove packaged charts
# must clean if any of the contents changed due to limitation of "build.rs"
clean:
	rm -rf pkg_sys;
	rm -rf pkg_app;

# Package a chart directory into a stable tgz name.
# Prefer `helm package` when helm is installed; fall back to tar so
# `include_dir!` embeds (and unit tests) still work without helm.
# Use `command -v helm` at recipe time (not make-parse time) so PATH from
# CI install steps is visible.
define package_chart
	set -e; \
	mkdir -p $(1); \
	rm -f $(1)/$(3); \
	if command -v helm >/dev/null 2>&1; then \
		echo "Packaging $(2) with helm (app-version=$(APP_VERSION))"; \
		helm package ./$(2) --app-version "$(APP_VERSION)" -d $(1); \
		cd $(1) && mv -f $(2)-*.tgz $(3); \
	else \
		echo "helm not found; packaging $(2) with tar into $(1)/$(3)"; \
		tar -czf $(1)/$(3) $(2); \
	fi; \
	test -s $(1)/$(3); \
	ls -la $(1)/$(3)
endef

# generate packaged chart for sys
pkg_sys:	streamfy-sys/*.* ../../VERSION
	$(call package_chart,pkg_sys,streamfy-sys,streamfy-chart-sys.tgz)

# generate packaged chart for app
pkg_app:	streamfy-app/*.* ../../VERSION
	$(call package_chart,pkg_app,streamfy-app,streamfy-chart-app.tgz)

package:	pkg_sys pkg_app
	echo "Packaged charts are in pkg_sys and pkg_app"
	@ls -la pkg_sys pkg_app
	@test -s pkg_sys/streamfy-chart-sys.tgz
	@test -s pkg_app/streamfy-chart-app.tgz
