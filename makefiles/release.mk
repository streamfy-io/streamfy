# Use the binary name produced by cargo
PUBLISH_BINARIES=streamfy streamfy-run streamfy-channel streamfy-test smdk fvm
PUBLISH_BINARIES_HUB=cdk

# CI has to set RELEASE=true to run commands that update public
#RELEASE?=false
ifneq ($(RELEASE),true)
DRY_RUN_ECHO=echo
#$(info Dry run mode - No public changes)
else
# When this is blank, commands will affect the public releases
DRY_RUN_ECHO=
#$(info Live mode - Public changes possible)
endif

GIT_COMMIT_SHA=$(shell git rev-parse HEAD)
STABLE_VERSION_TAG?=stable
REPO_VERSION?=$(shell cat VERSION)
DEV_VERSION_TAG?=$(REPO_VERSION)-$(GIT_COMMIT_SHA)
CHANNEL_TAG?=stable

# VERSION is used mostly by build, so here we
# use CHANNEL_TAG to override value of VERSION for CI
ifeq ($(CHANNEL_TAG), stable)
VERSION:=$(STABLE_VERSION_TAG)
#$(info Working with channel $(CHANNEL_TAG) version: $(VERSION))
else ifeq ($(CHANNEL_TAG), latest)
VERSION:=$(DEV_VERSION_TAG)
#$(info Working with channel $(CHANNEL_TAG) version: $(VERSION))
else
VERSION:=$(CHANNEL_TAG)
endif
#$(info Working with version: $(VERSION))

DOCKER_USERNAME?=test-docker-user
DOCKER_PASSWORD?=test-docker-pass
DOCKER_IMAGE_TAG?=$(REPO_VERSION)
#$(info Docker image tag: $(DOCKER_IMAGE_TAG))

GH_TOKEN?=
GH_RELEASE_TAG?=dev

ifeq ($(PRE_RELEASE),true)
GH_PRE_RELEASE_FLAG=--prerelease
else
GH_PRE_RELEASE_FLAG=
endif

# Allow using local `gh` auth token for local testing
#ifeq ($(CI), true)
#ifndef GH_TOKEN
#$(error GH_TOKEN required in CI)
#endif
#endif

DIRNAME?=
TARGET?=
PACKAGE?=
ARTIFACT?=

# Streamfy Cloud Version used to publish pkgsets
STREAMFY_CLOUD_VERSION?=stable

#### Testing only

get-version:
	echo $(VERSION)

get-tag:
	echo $(DEV_VERSION_TAG)

clean-publish:
	rm -vf *.zip *.tgz *.exe
	rm -vrf streamfy-* streamfy.* smdk-*
	rm -vf /tmp/release_notes /tmp/cd_dev_latest.txt

#fix-latest-channel:
#	# Find the last git sha from master
#	# Re-set all tags to use the version and sha from that commit
#### End testing


docker-check-image-exists:
	if [ $(lastword $(shell docker pull --quiet streamfy-io/streamfy:$(DOCKER_IMAGE_TAG); echo $$?)) -eq 0 ]; then \
		echo Image tag already exists; \
		exit 0; \
	else \
		echo Image tag does not exist; \
		exit 1; \
	fi

# Get Streamfy VERSION from Github, provided a given git SHA
docker-create-manifest:
	$(DRY_RUN_ECHO) docker manifest create "ghcr.io/streamfy-io/streamfy:$(DOCKER_IMAGE_TAG)" \
		"ghcr.io/streamfy-io/streamfy:$(DEV_VERSION_TAG)-amd64" \
		"ghcr.io/streamfy-io/streamfy:$(DEV_VERSION_TAG)-arm64v8"

docker-push-manifest: docker-create-manifest
	$(DRY_RUN_ECHO) docker manifest push "ghcr.io/streamfy-io/streamfy:$(DOCKER_IMAGE_TAG)"

# Create latest development Streamfy image
docker-create-manifest-dev: DOCKER_IMAGE_TAG=latest
docker-create-manifest-dev: docker-create-manifest

# Push docker manifest
docker-push-manifest-dev: DOCKER_IMAGE_TAG=latest
docker-push-manifest-dev: docker-create-manifest-dev docker-push-manifest

# Uses $(VERSION)
curl-install-streamfy:
	curl -fsS https://raw.githubusercontent.com/streamfy-io/streamfy/master/install.sh | bash

install-streamfy-stable: VERSION=stable
install-streamfy-stable: curl-install-streamfy

install-streamfy-latest: VERSION=latest
install-streamfy-latest: curl-install-streamfy

install-streamfy-package: STREAMFY_BIN=$(HOME)/.streamfy/bin/streamfy
install-streamfy-package:
	# temporarily remove deadlock on streamfy-package install
	# $(STREAMFY_BIN) install streamfy-package
	mkdir -p ${HOME}/.streamfy/extensions
	curl https://packages.streamfy.io/v1/packages/streamfy/streamfy-package/0.1.9/x86_64-unknown-linux-musl/streamfy-package \
	-o ${HOME}/.streamfy/extensions/streamfy-package
	chmod +x ${HOME}/.streamfy/extensions/streamfy-package

# Requires GH_TOKEN set or `gh auth login`
download-streamfy-release:
	$(DRY_RUN_ECHO) gh release download $(GH_RELEASE_TAG) -R streamfy-io/streamfy --skip-existing

unzip-gh-release-artifacts: download-streamfy-release
	@echo "unzip stuff"
	@$(foreach bin, $(wildcard *.zip), \
		printf "\n"; \
		export DIRNAME=$(basename $(bin)); \
		unzip -u -d $$DIRNAME $(bin); \
	)

# Publish artifacts from GH Releases to Streamfy Packages
#
# Artifacts from GH Releases look like this:
#
# ./
#   ARTIFACT-TARGET.zip, such as:
#   streamfy-x86_64-unknown-linux-musl.zip
#   streamfy-aarch64-unknown-linux-musl.zip
#   streamfy-x86_64-apple-darwin.zip
#
# Here, we extract each zip into dirs with the same name.
# Then, we get the TARGET from the `.target` file inside.
#
# ./
#   ARTIFACT-TARGET.zip
#   ARTIFACT-TARGET/
#     ARTIFACT
#     .target
#   streamfy-x86_64-unknown-linux-musl.zip
#   streamfy-x86_64-unknown-linux-musl/
#     streamfy
#     .target
publish-artifacts: PUBLIC_VERSION=$(subst -$(GIT_COMMIT_SHA),+$(GIT_COMMIT_SHA),$(VERSION))
publish-artifacts: install-streamfy-package unzip-gh-release-artifacts
	@echo "package stuff"
	$(foreach bin, $(wildcard *.zip), \
		printf "\n"; \
		export DIRNAME=$(basename $(bin)); \
		export TARGET=$(shell cat $(basename $(bin))/.target); \
		export PACKAGE=$(subst -$(shell cat $(basename $(bin))/.target), ,$(basename $(bin))); \
		export ARTIFACT=$(abspath $$DIRNAME/$$PACKAGE); \
		$(DRY_RUN_ECHO) $(STREAMFY_BIN) package publish \
			--package=$(subst .exe, ,$(subst -$(shell cat $(basename $(bin))/.target), ,$(basename $(bin)))) \
			--version=$(PUBLIC_VERSION) \
			--target=$$TARGET \
			$$ARTIFACT || true; \
	)

publish-artifacts-hub: PUBLIC_VERSION=$(subst -$(GIT_COMMIT_SHA),+$(GIT_COMMIT_SHA),$(VERSION))
publish-artifacts-hub: unzip-gh-release-artifacts
	@echo "Publish to hub"
	$(foreach bin, $(PUBLISH_BINARIES_HUB), \
		$(foreach zipf, $(wildcard ${bin}*.zip), \
			printf "\n"; \
			export DIRNAME=$(basename $(zipf)); \
			export TARGET=$(shell cat $(basename $(zipf))/.target); \
			export PACKAGE=$(subst -$(shell cat $(basename $(zipf))/.target), ,$(basename $(zipf))); \
			export ARTIFACT=$(abspath $$DIRNAME/$$PACKAGE); \
			$(DRY_RUN_ECHO) actions/upload-bpkg.sh $$ARTIFACT $$TARGET ${CHANNEL}; \
		) \
	)

publish-artifacts-dev-hub: CHANNEL=latest
publish-artifacts-dev-hub: publish-artifacts-hub

publish-artifacts-dev: VERSION=$(DEV_VERSION_TAG)
publish-artifacts-dev: publish-artifacts publish-artifacts-dev-hub

publish-artifacts-stable-hub: CHANNEL=stable
publish-artifacts-stable-hub: publish-artifacts-hub

publish-artifacts-stable: VERSION=$(REPO_VERSION)
publish-artifacts-stable: publish-artifacts publish-artifacts-stable-hub

# Need to ensure that version is always a semver
# Version convention is different here. Notice the `+`
bump-streamfy: STREAMFY_BIN=$(HOME)/.streamfy/bin/streamfy
bump-streamfy: PUBLIC_VERSION?=$(subst -$(GIT_COMMIT_SHA),+$(GIT_COMMIT_SHA),$(VERSION))
bump-streamfy: install-streamfy-package
# This is gonna end up echoing twice when RELEASE != true
	$(DRY_RUN_ECHO) $(STREAMFY_BIN) package bump $(CHANNEL_TAG) $(PUBLIC_VERSION)
	@$(foreach bin, $(PUBLISH_BINARIES), \
		printf "\n"; \
		echo $(STREAMFY_BIN) package tag $(bin):$(PUBLIC_VERSION) --allow-missing-targets --tag=$(CHANNEL_TAG) --force; \
		$(DRY_RUN_ECHO) $(STREAMFY_BIN) package tag $(bin):$(PUBLIC_VERSION) --allow-missing-targets --tag=$(CHANNEL_TAG) --force; \
	)

# publishes pkgset for stable e.g. 0.11.0
# uses STREAMFY_CLOUD_VERSION
publish-pkgset: PKGSET_NAME=${REPO_VERSION}
publish-pkgset: STREAMFY_VERSION=${REPO_VERSION}
publish-pkgset:
	./actions/publish-pkgset.sh

bump-streamfy-stable: CHANNEL_TAG=stable
bump-streamfy-stable: VERSION=$(REPO_VERSION)
# publishes pkgset for "stable"
bump-streamfy-stable: PKGSET_NAME=stable
bump-streamfy-stable: STREAMFY_VERSION=${VERSION}
bump-streamfy-stable: bump-streamfy publish-pkgset
	./actions/publish-pkgset.sh

bump-streamfy-latest: CHANNEL_TAG=latest
bump-streamfy-latest: VERSION=$(subst -$(GIT_COMMIT_SHA),+$(GIT_COMMIT_SHA),$(DEV_VERSION_TAG))
# publishes pkgset for "latest"
bump-streamfy-latest: PKGSET_NAME=latest
bump-streamfy-latest: STREAMFY_VERSION=${VERSION}
bump-streamfy-latest: STREAMFY_CLOUD_VERSION=latest
bump-streamfy-latest: bump-streamfy
	./actions/publish-pkgset.sh

update-public-installer-script-s3:
	$(DRY_RUN_ECHO) aws s3 cp ./install.sh s3://packages.streamfy.io/v1/install.sh --acl public-read

latest-cd-dev-status:
	gh api /repos/{owner}/{repo}/actions/workflows/cd_dev.yaml/runs | jq .workflow_runs[0] > /tmp/cd_dev_latest.txt
	@echo "Latest CD_Dev run: $$( cat /tmp/cd_dev_latest.txt | jq .html_url | tr -d '"')"

	@if [ $$(cat /tmp/cd_dev_latest.txt | jq .conclusion | tr -d '"') = success ]; then \
		echo ✅ Most recent CD_Dev run passed; \
		exit 0; \
	else \
		echo ❌ Most recent CD_Dev run failed; \
		exit 1; \
	fi

build-release-notes:
	rm --verbose --force /tmp/release_notes
	touch /tmp/release_notes
	echo "# Release Notes" >> /tmp/release_notes
	export VERSION=$(shell cat VERSION)
	cat CHANGELOG.md | sed -e '/./{H;$$!d;}' -e "x;/##\ Platform\ Version\ $$VERSION/"'!d;' >> /tmp/release_notes

	# Replace UNRELEASED w/ date YYYY-MM-dd
	export TZ=":America/Los_Angeles"
	cat /tmp/release_notes | sed -i "s/UNRELEASED/$(shell date +%F)/" /tmp/release_notes

	# Print the release notes to stdout
	cat /tmp/release_notes

create-gh-release: download-streamfy-release build-release-notes
	$(DRY_RUN_ECHO) gh release create -R streamfy-io/streamfy \
		$(GH_PRE_RELEASE_FLAG) \
		--title="v$(VERSION)" \
		-F /tmp/release_notes \
		"v$(VERSION)" \
		$(wildcard *.zip *.tgz)
