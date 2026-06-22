#!/usr/bin/env bats

TEST_HELPER_DIR="$BATS_TEST_DIRNAME/../../test_helper"
export TEST_HELPER_DIR

load "$TEST_HELPER_DIR"/tools_check.bash
load "$TEST_HELPER_DIR"/streamfy_dev.bash
load "$TEST_HELPER_DIR"/bats-support/load.bash
load "$TEST_HELPER_DIR"/bats-assert/load.bash

# Add at least one of each type of resource into the cluster
setup_file() {

    # metadata
    STREAMFY_METADATA_DIR="$HOME/.streamfy/data/metadata"
    export STREAMFY_METADATA_DIR
    debug_msg "Streamfy Metadata Directory: $STREAMFY_METADATA_DIR"

    # topic
    run timeout 15s "$STREAMFY_BIN" topic create "$(random_string)"
}

# Delete the cluster
@test "Delete the cluster" {
    if [ "$STREAMFY_CLI_RELEASE_CHANNEL" == "dev" -a "$STREAMFY_CLUSTER_RELEASE_CHANNEL" == "stable" ]; then
        skip "don't run on stable cluster version and dev cli version" # remove this when installation type is available on stable
    fi

    run bash -c "$STREAMFY_BIN cluster delete --force || $STREAMFY_BIN cluster delete"
    assert_success

    run test -d $STREAMFY_METADATA_DIR
    assert_failure
}

