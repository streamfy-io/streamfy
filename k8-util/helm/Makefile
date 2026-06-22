MAKE_DIR = $(dir $(realpath $(firstword $(MAKEFILE_LIST))))
VERSION := $(shell cat $(MAKE_DIR)/../../VERSION)
APP_VERSION = $(VERSION)

# remove packaged charts
# must clean if any of the contents changed due to limitation of "build.rs"
clean:
	rm -rf pkg_sys;
	rm -rf pkg_app;

# generate packaged chart for sys
pkg_sys:	streamfy-sys/*.* ../../VERSION
	mkdir -p pkg_sys;
	helm package ./streamfy-sys --app-version $(APP_VERSION) -d pkg_sys
	cd pkg_sys;mv streamfy-sys-* streamfy-chart-sys.tgz

# generate packaged chart for app
pkg_app:	streamfy-app/*.* ../../VERSION
	mkdir -p pkg_app
	helm package ./streamfy-app --app-version $(APP_VERSION) -d pkg_app
	cd pkg_app;mv streamfy-app-* streamfy-chart-app.tgz

package:	pkg_sys pkg_app
	echo "Packaged charts are in pkg_sys and pkg_app"