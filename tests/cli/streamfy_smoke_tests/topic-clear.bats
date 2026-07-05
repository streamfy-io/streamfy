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
    debug_msg "Topic name: $TOPIC_NAME"
    run timeout 15s "$STREAMFY_BIN" topic create "$TOPIC_NAME"
    assert_success
}

teardown_file() {
    run timeout 15s "$STREAMFY_BIN" topic delete "$TOPIC_NAME"
}

# Confirmation is required by default; declining aborts without clearing.
@test "topic clear aborts without confirmation" {
    # Produce must use `run` if we assert_success (sets $status/$output).
    run bash -c "echo old-record | timeout 15s \"$STREAMFY_BIN\" produce \"$TOPIC_NAME\""
    assert_success

    run bash -c "echo n | timeout 15s \"$STREAMFY_BIN\" topic clear \"$TOPIC_NAME\""
    assert_success
    assert_output --partial "Aborted"

    # -B and -T conflict; other smoke tests use -B -d only.
    run timeout 15s "$STREAMFY_BIN" consume "$TOPIC_NAME" -B -d
    assert_success
    assert_output --partial "old-record"
}

# --yes bypasses the interactive prompt.
@test "topic clear --yes clears without prompt" {
    TOPIC_YES=$(random_string)
    run timeout 15s "$STREAMFY_BIN" topic create "$TOPIC_YES"
    assert_success

    run bash -c "echo to-be-cleared | timeout 15s \"$STREAMFY_BIN\" produce \"$TOPIC_YES\""
    assert_success

    run timeout 15s "$STREAMFY_BIN" topic clear "$TOPIC_YES" --yes
    assert_success
    assert_output --partial "topic \"$TOPIC_YES\" cleared"

    run timeout 15s "$STREAMFY_BIN" consume "$TOPIC_YES" -B -d
    assert_success
    refute_output --partial "to-be-cleared"

    run timeout 15s "$STREAMFY_BIN" topic delete "$TOPIC_YES"
}

# -y is an alias for --yes.
@test "topic clear -y clears without prompt" {
    TOPIC_Y=$(random_string)
    run timeout 15s "$STREAMFY_BIN" topic create "$TOPIC_Y"
    assert_success

    run bash -c "echo short-flag | timeout 15s \"$STREAMFY_BIN\" produce \"$TOPIC_Y\""
    assert_success

    run timeout 15s "$STREAMFY_BIN" topic clear "$TOPIC_Y" -y
    assert_success
    assert_output --partial "topic \"$TOPIC_Y\" cleared"

    run timeout 15s "$STREAMFY_BIN" topic delete "$TOPIC_Y"
}

# End-to-end: data cleared, consumer offsets preserved, new produce/consume works.
@test "topic clear removes records preserves offsets and allows new produce consume" {
    TOPIC_E2E=$(random_string)
    CONSUMER_NAME=$(random_string)

    run timeout 15s "$STREAMFY_BIN" topic create "$TOPIC_E2E"
    assert_success

    run bash -c "echo record-one | timeout 15s \"$STREAMFY_BIN\" produce \"$TOPIC_E2E\""
    assert_success
    run bash -c "echo record-two | timeout 15s \"$STREAMFY_BIN\" produce \"$TOPIC_E2E\""
    assert_success

    # Establish a consumer offset by reading from the beginning.
    run timeout 15s "$STREAMFY_BIN" consume "$TOPIC_E2E" --consumer "$CONSUMER_NAME" -B -d
    assert_success
    assert_output --partial "record-one"
    assert_output --partial "record-two"

    OFFSET_BEFORE=$("$STREAMFY_BIN" consumer list -O json | jq ".[] | select(.consumer_id == \"$CONSUMER_NAME\") | .offset")
    assert [ -n "$OFFSET_BEFORE" ]
    assert [ "$OFFSET_BEFORE" != "null" ]

    # Topic must still exist with same name after clear.
    run timeout 15s "$STREAMFY_BIN" topic clear "$TOPIC_E2E" -y
    assert_success
    assert_output --partial "topic \"$TOPIC_E2E\" cleared"

    run timeout 15s "$STREAMFY_BIN" topic list
    assert_success
    assert_output --partial "$TOPIC_E2E"

    # Previously stored records are no longer readable from the beginning.
    run timeout 15s "$STREAMFY_BIN" consume "$TOPIC_E2E" -B -d
    assert_success
    refute_output --partial "record-one"
    refute_output --partial "record-two"

    # Consumer offsets remain unchanged (not deleted with the topic data).
    OFFSET_AFTER=$("$STREAMFY_BIN" consumer list -O json | jq ".[] | select(.consumer_id == \"$CONSUMER_NAME\") | .offset")
    assert [ "$OFFSET_AFTER" == "$OFFSET_BEFORE" ]

    # New records can be produced and consumed normally.
    run bash -c "echo record-after-clear | timeout 15s \"$STREAMFY_BIN\" produce \"$TOPIC_E2E\""
    assert_success

    run timeout 15s "$STREAMFY_BIN" consume "$TOPIC_E2E" -B -d
    assert_success
    assert_output --partial "record-after-clear"

    run timeout 15s "$STREAMFY_BIN" consumer delete "$CONSUMER_NAME"
    run timeout 15s "$STREAMFY_BIN" topic delete "$TOPIC_E2E"
}

# Accepting the confirmation prompt proceeds with clear.
@test "topic clear proceeds when user confirms" {
    TOPIC_CONFIRM=$(random_string)
    run timeout 15s "$STREAMFY_BIN" topic create "$TOPIC_CONFIRM"
    assert_success

    run bash -c "echo confirm-me | timeout 15s \"$STREAMFY_BIN\" produce \"$TOPIC_CONFIRM\""
    assert_success

    run bash -c "echo y | timeout 15s \"$STREAMFY_BIN\" topic clear \"$TOPIC_CONFIRM\""
    assert_success
    assert_output --partial "topic \"$TOPIC_CONFIRM\" cleared"

    run timeout 15s "$STREAMFY_BIN" consume "$TOPIC_CONFIRM" -B -d
    assert_success
    refute_output --partial "confirm-me"

    run timeout 15s "$STREAMFY_BIN" topic delete "$TOPIC_CONFIRM"
}
