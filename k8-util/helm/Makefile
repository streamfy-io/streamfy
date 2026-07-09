MAKE_DIR = $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
VERSION := $(shell cat $(MAKE_DIR)/../../VERSION)
APP_VERSION = $(VERSION)
HELM ?= $(shell command -v helm 2>/dev/null)

# remove packaged charts
# must clean if any of the contents changed due to limitation of "build.rs"
clean:
	rm -rf pkg_sys;
	rm -rf pkg_app;

# Package a chart directory into a stable tgz name.
# Prefer `helm package` when helm is installed; fall back to tar so
# `include_dir!` embeds (and unit tests) still work without helm.
define package_chart
	mkdir -p $(1)
	rm -f $(1)/$(3)
	if [ -n "$(HELM)" ]; then \
		$(HELM) package ./$(2) --app-version $(APP_VERSION) -d $(1); \
		cd $(1) && mv $(2)-* $(3); \
	else \
		echo "helm not found; packaging $(2) with tar into $(1)/$(3)"; \
		tar -czf $(1)/$(3) $(2); \
	fi
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