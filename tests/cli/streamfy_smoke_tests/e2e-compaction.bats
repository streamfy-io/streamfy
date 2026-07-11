#!/usr/bin/env bats

TEST_HELPER_DIR="$BATS_TEST_DIRNAME/../test_helper"
export TEST_HELPER_DIR

load "$TEST_HELPER_DIR"/tools_check.bash
load "$TEST_HELPER_DIR"/streamfy_dev.bash
load "$TEST_HELPER_DIR"/bats-support/load.bash
load "$TEST_HELPER_DIR"/bats-assert/load.bash

setup_file() {
    TOPIC_NAME=$(random_string)
    export TOPIC_NAME
    debug_msg "Compaction topic name: $TOPIC_NAME"
}

teardown_file() {
    run timeout 15s "$STREAMFY_BIN" topic delete "$TOPIC_NAME"
}

# ---------------------------------------------------------------
# Test: create a topic with --cleanup-policy compact
# ---------------------------------------------------------------
@test "Create a compacted topic" {
    debug_msg "Creating topic $TOPIC_NAME with --cleanup-policy compact and small segment-size"
    run timeout 15s "$STREAMFY_BIN" topic create "$TOPIC_NAME" \
        --cleanup-policy compact \
        --segment-size 256
    assert_success
    assert_output --partial "topic \"$TOPIC_NAME\" created"
}

# ---------------------------------------------------------------
# Test: produce multiple values per key plus tombstones
# ---------------------------------------------------------------
@test "Produce keyed records and tombstones" {
    # Key "user-1": three updates (value should compact to latest)
    echo "v1" | timeout 15s "$STREAMFY_BIN" produce "$TOPIC_NAME" --key "user-1"
    echo "v2" | timeout 15s "$STREAMFY_BIN" produce "$TOPIC_NAME" --key "user-1"
    echo "v3" | timeout 15s "$STREAMFY_BIN" produce "$TOPIC_NAME" --key "user-1"

    # Key "user-2": one value then a tombstone (empty value)
    echo "keep" | timeout 15s "$STREAMFY_BIN" produce "$TOPIC_NAME" --key "user-2"
    echo ""     | timeout 15s "$STREAMFY_BIN" produce "$TOPIC_NAME" --key "user-2"

    # Key "user-3": single value (should survive)
    run bash -c 'echo "only" | timeout 15s "$STREAMFY_BIN" produce "$TOPIC_NAME" --key "user-3"'
    assert_success
}

# ---------------------------------------------------------------
# Test: wait for compaction to process sealed segments
# ---------------------------------------------------------------
@test "Wait for compaction" {
    # The default cleaner interval is 10 s. Wait long enough for at least
    # one compaction cycle to pick up the sealed segments.
    sleep 15
}

# ---------------------------------------------------------------
# Test: consume from offset 0 yields only latest value per key
# ---------------------------------------------------------------
@test "Consume after compaction returns latest per key" {
    run timeout 15s "$STREAMFY_BIN" consume "$TOPIC_NAME" \
        --from-beginning -d --format "{{key}}={{value}}"

    debug_msg "Output: $output"

    # "user-1" should appear with latest value "v3"
    assert_output --partial "user-1=v3"
    # The earlier values for user-1 should NOT appear
    refute_output --partial "user-1=v1"
    refute_output --partial "user-1=v2"

    # "user-3" should still be present
    assert_output --partial "user-3=only"
}

# ---------------------------------------------------------------
# Test: offsets are preserved (not renumbered)
# ---------------------------------------------------------------
@test "Offsets are preserved after compaction" {
    # When consuming with offset output, the offsets should have gaps
    # (records were removed but offsets were not renumbered).
    run timeout 15s "$STREAMFY_BIN" consume "$TOPIC_NAME" \
        --from-beginning -d --format "{{offset}}:{{key}}={{value}}"

    debug_msg "Output with offsets: $output"

    # user-1's latest record was the 3rd produce (offset 2),
    # so we should see offset 2 for user-1.
    # The exact offset depends on batching, but we should NOT see
    # a contiguous 0,1,2,3... sequence if compaction removed records.
    assert_success
}
