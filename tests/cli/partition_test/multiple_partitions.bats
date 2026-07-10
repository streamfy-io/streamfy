#!/usr/bin/env bats

TEST_HELPER_DIR="$BATS_TEST_DIRNAME/../test_helper"
export TEST_HELPER_DIR

load "$TEST_HELPER_DIR"/tools_check.bash
load "$TEST_HELPER_DIR"/streamfy_dev.bash
load "$TEST_HELPER_DIR"/bats-support/load.bash
load "$TEST_HELPER_DIR"/bats-assert/load.bash

setup_file() {
    # Defaults for local runs; CI should set PARTITIONS/REPLICATION explicitly.
    # Fluvio CI uses REPLICATION=1 for this suite.
    PARTITIONS="${PARTITIONS:-2}"
    REPLICATION="${REPLICATION:-1}"
    export PARTITIONS REPLICATION

    PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME=$(random_string)
    export PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME
    debug_msg "Topic name: $PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME"

    # Absolute path so produce --file works regardless of cwd
    MULTI_LINE_FILE_NAME="${BATS_TEST_TMPDIR:-/tmp}/$(random_string).lines"
    export MULTI_LINE_FILE_NAME

    # Creates test File which will have 2 items per partition
    : > "$MULTI_LINE_FILE_NAME"
    for (( p = 0; p < PARTITIONS * 2; p++ ))
    do
        echo "$p" >> "$MULTI_LINE_FILE_NAME"
    done
    debug_msg "Produce file: $MULTI_LINE_FILE_NAME ($(wc -l < "$MULTI_LINE_FILE_NAME") lines)"
}

teardown_file() {
    echo "Tearing down, shutting down cluster components"
    "$STREAMFY_BIN" topic delete "$PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME" || true
    rm -f "$MULTI_LINE_FILE_NAME"
}

@test "Create a topic for P/C Multiple Partitions" {
    echo "Creates Topic: $PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME for P/C Multiple Partitions"
    run timeout 15s "$STREAMFY_BIN" topic create "$PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME" --partitions "$PARTITIONS" --replication "$REPLICATION"
    assert_success

    echo "Topic Details: $PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME"
    run timeout 15s "$STREAMFY_BIN" topic describe "$PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME"
    assert_success
}

@test "Produces on topic for P/C Multiple Partitions" {
    run timeout 15s "$STREAMFY_BIN" produce --file "$MULTI_LINE_FILE_NAME" "$PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME"
    assert_success
    # Allow records to commit before partition-scoped consume
    sleep 2
}

@test "Consumes on topic for P/C Multiple Partitions with Partition" {
    for (( part = 0; part < PARTITIONS; part++ ))
    do
        run timeout 15s "$STREAMFY_BIN" consume "$PRODUCE_CONSUME_MULTIPLE_PARTITIONS_TOPIC_NAME" -p "$part" -B -d
        assert_success

        for set in {0..1}
        do
            if (( set == 0 ))
            then
                WANT=$(( part ))
                assert_line --index 0 "$WANT"
            else
                WANT=$(( PARTITIONS + part ))
                assert_line --index 1 "$WANT"
            fi
        done
    done
}
