#!/usr/bin/env bash

#set -exu
set -eu

readonly STREAMFY_BIN=~/.streamfy/bin/streamfy
readonly STREAMFY_VERSION_CHECK=${1?Pass in expected version in pos 1}
readonly STREAMFY_COMMIT_CHECK=${2?Pass in expected commit in pos 2}

# This function should always run first
function validate_installer_output() {
    # Validate the installer output returns the expected version
    curl -fsS https://raw.githubusercontent.com/streamfy-io/streamfy/master/install.sh | bash | tee /tmp/installer.output

    INSTALLED_STREAMFY_VERSION=$(cat /tmp/installer.output | grep "Downloading Streamfy" | awk '{print $5}' | tr -d '[:space:]')
    EXPECTED_STREAMFY_VERSION=$STREAMFY_VERSION_CHECK

    if [ "$INSTALLED_STREAMFY_VERSION" = "$EXPECTED_STREAMFY_VERSION" ]; then
      echo "✅ Version reported by installer: $INSTALLED_STREAMFY_VERSION";
    else
      echo "❌ Version install check failed";
      echo "Version reported by installer: $INSTALLED_STREAMFY_VERSION";
      echo "Expected version: $EXPECTED_STREAMFY_VERSION";
      exit 1;
    fi
}

function validate_streamfy_sha256() {
    # Validate streamfy binary checksum
    INSTALLED_STREAMFY_SHASUM=$($STREAMFY_BIN version | grep SHA256 | awk '{print $5}')
    EXPECTED_STREAMFY_SHASUM=$(shasum -a 256 $STREAMFY_BIN  | awk '{print $1}' | tr -d '[:space:]')

    if [ "$INSTALLED_STREAMFY_SHASUM" = "$EXPECTED_STREAMFY_SHASUM" ]; then
      echo "✅ Sha256 check by streamfy version passed: $INSTALLED_STREAMFY_SHASUM";
    else
      echo "❌ streamfy version reported unexpected shasum";
      echo "Shasum reported by streamfy version: $INSTALLED_STREAMFY_SHASUM";
      echo "Expected streamfy version: $EXPECTED_STREAMFY_SHASUM";
      exit 1;
    fi
}

function validate_streamfy_commit() {
    # Validate streamfy binary commit 
    INSTALLED_STREAMFY_COMMIT=$(streamfy version | grep Commit | awk '{print $4}' | tr -d '[:space:]')
    EXPECTED_STREAMFY_COMMIT=$STREAMFY_COMMIT_CHECK
    if [ "$INSTALLED_STREAMFY_COMMIT" = "$EXPECTED_STREAMFY_COMMIT" ]; then
      echo "✅ Installed streamfy commit check passed: $INSTALLED_STREAMFY_COMMIT";
    else
      echo "❌ Installed streamfy commit check failed";
      echo "Commit reported by streamfy version: $INSTALLED_STREAMFY_COMMIT";
      echo "Expected commit: $EXPECTED_STREAMFY_COMMIT";
      exit 1;
    fi
}

# Validate streamfy-run version in docker image
function validate_docker_image() {
    # Download the docker image
    EXPECTED_STREAMFY_RUN_VERSION=$STREAMFY_VERSION_CHECK

    # Validate that the docker image has the correct Streamfy binaries
    docker pull streamfy-io/streamfy:$EXPECTED_STREAMFY_RUN_VERSION
    DOCKER_STREAMFY_RUN_VERSION=$(docker run streamfy-io/streamfy:$EXPECTED_STREAMFY_RUN_VERSION sh -c "/streamfy-run --version" | awk '{print $2}' | tr -d '[:space:]')

    if [ "$DOCKER_STREAMFY_RUN_VERSION" = "$EXPECTED_STREAMFY_RUN_VERSION" ]; then
      echo "✅ Docker streamfy run version check passed: $EXPECTED_STREAMFY_RUN_VERSION";
    else
      echo "❌ Docker streamfy run version check failed";
      echo "Version reported by streamfy-run: $DOCKER_STREAMFY_RUN_VERSION";
      echo "Expected version: $EXPECTED_STREAMFY_RUN_VERSION";
      exit 1;
    fi
}

function main() {
    validate_installer_output;
    validate_streamfy_sha256;
    validate_streamfy_commit;
    validate_docker_image;
}

main;
