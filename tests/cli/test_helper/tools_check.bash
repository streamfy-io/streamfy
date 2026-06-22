# Resolve path to `streamfy` binary instead of expecting it in PATH
# Search order: $STREAMFY_BIN, in PATH, current directory, home directory
main() {
    # Take in override to test_helper directory
    TEST_HELPER_DIR=${TEST_HELPER_DIR:-./test_helper}
    export TEST_HELPER_DIR

    # BATS_TEST_RETRIES is set to default after bats started therefore we set it here
    BATS_TEST_RETRIES=${CLI_TEST_RETRIES:-0}
    export BATS_TEST_RETRIES

    check_load_bats_libraries;
    check_streamfy_bin_path;
    check_timeout_bin;
}

function check_streamfy_bin_path() {

    if [[ -n $STREAMFY_BIN ]]; then
        if [[ -n $DEBUG ]]; then
            echo "# DEBUG: found: STREAMFY_BIN was defined"
        fi
        _set_streamfy_bin_path_then_exit "$STREAMFY_BIN";
    elif which streamfy; then
        if [[ -n $DEBUG ]]; then
            echo "# DEBUG: found: streamfy in PATH"
        fi
        _set_streamfy_bin_path_then_exit "$(which streamfy)";
    elif test -f "$(pwd)/streamfy"; then
        if [[ -n $DEBUG ]]; then
            echo "# DEBUG: found: streamfy in current directory"
        fi
        _set_streamfy_bin_path_then_exit "$(pwd)/streamfy";
    elif test -f "$HOME/.streamfy/bin/streamfy"; then
        if [[ -n $DEBUG ]]; then
            echo "# DEBUG: found: streamfy in home directory"
        fi
        _set_streamfy_bin_path_then_exit "$HOME/.streamfy/bin/streamfy";
    fi
}

function _set_streamfy_bin_path_then_exit() {
    STREAMFY_BIN=$1
    export STREAMFY_BIN
    if [[ -n $DEBUG ]]; then
        echo "# DEBUG: Streamfy binary path: $STREAMFY_BIN"
    fi

}

function check_streamfy_cluster() {
    if [[ -n $DEBUG ]]; then
        echo "# DEBUG: Attempting to start cluster with streamfy bin @ $STREAMFY_BIN" >&3
    fi
    run "$STREAMFY_BIN" cluster start
}

# Make sure Bats-core helper libraries are installed
function check_load_bats_libraries() {
    # Look for bats-support, bats-assert, bats-file
    # If not there, try to clone it into place

    if ! test -d "$TEST_HELPER_DIR/bats-support"; then
        echo "# Installing bats-support in $TEST_HELPER_DIR"
        git clone https://github.com/bats-core/bats-support "$TEST_HELPER_DIR/bats-support"
    fi

    if ! test -d "$TEST_HELPER_DIR/bats-assert"; then
        echo "# Installing bats-assert in $TEST_HELPER_DIR"
        git clone https://github.com/bats-core/bats-assert "$TEST_HELPER_DIR/bats-assert"
    fi
}

function check_timeout_bin() {
    if ! which timeout; then
        echo "# \`timeout\` not in PATH" >&3

        if [[ $(uname) == "Darwin" ]]; then
            echo "# run \`brew install coreutils\` to install"
        fi

        false
    fi
}

function wait_for_line_in_file() {
    LINE="$1"
    FILE="$2"
    MAX_SECONDS="${3:-30}" # 30 seconds default value

    echo "Waiting for file $FILE containing $LINE"

    ELAPSED=0;
    until grep -q "$LINE" "$FILE"
    do
      sleep 1
      let ELAPSED=$ELAPSED+1
      if [[ $ELAPSED -ge MAX_SECONDS ]]
      then
        echo "timeout $MAX_SECONDS seconds elapsed"
        exit 1
      fi
    done
    echo "Done waiting for file $FILE containing $LINE"
}



main;
