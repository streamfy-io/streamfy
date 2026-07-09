#!/usr/bin/env bats

TEST_HELPER_DIR="$BATS_TEST_DIRNAME/../test_helper"
export TEST_HELPER_DIR

load "$TEST_HELPER_DIR"/tools_check.bash
load "$TEST_HELPER_DIR"/streamfy_dev.bash
load "$TEST_HELPER_DIR"/bats-support/load.bash
load "$TEST_HELPER_DIR"/bats-assert/load.bash

setup_file() {
    # Tests in this file are executed in order and rely on the previous test
    # to be successful.

    export STREAMFY_CI_CONTEXT="ci"

    # Retrieves the latest stable version from the GitHub API and removes the
    # `v` prefix from the tag name.
    STABLE_VERSION=$(curl "https://api.github.com/repos/streamfy-io/streamfy/releases/latest" | jq -er '.tag_name' | cut -c2-)
    export STABLE_VERSION
    debug_msg "Stable Version: $STABLE_VERSION"
    
    STREAMFY_RUN_STABLE_VERSION="$STABLE_VERSION"
    export STREAMFY_RUN_STABLE_VERSION

    # Fetches current SVM Stable Version from GitHub releases
    # Use the Streamfy release tag as the SVM stable version indicator for tests
    SVM_VERSION_STABLE="$STABLE_VERSION"
    export SVM_VERSION_STABLE
    debug_msg "SVM Stable Version: $SVM_VERSION_STABLE"

    SVM_UPDATE_CUSTOM_VERSION="0.18.0"
    export SVM_UPDATE_CUSTOM_VERSION
    debug_msg "SVM Update Custom Version: $SVM_UPDATE_CUSTOM_VERSION"

    # The directory where SVM files live
    SVM_HOME_DIR="$HOME/.svm"
    export SVM_HOME_DIR
    debug_msg "SVM Home Directory: $SVM_HOME_DIR"

    # The directory where SVM stores the downloaded versions
    VERSIONS_DIR="$SVM_HOME_DIR/versions"
    export VERSIONS_DIR
    debug_msg "Versions Directory: $VERSIONS_DIR"

    # The path to the Settings Toml file
    SETTINGS_TOML_PATH="$SVM_HOME_DIR/settings.toml"
    export SETTINGS_TOML_PATH
    debug_msg "Settings Toml Path: $SETTINGS_TOML_PATH"

    STATIC_VERSION="0.10.15"
    export STATIC_VERSION
    debug_msg "Static Version: $STATIC_VERSION"

    STREAMFY_HOME_DIR="$HOME/.streamfy"
    export STREAMFY_HOME_DIR
    debug_msg "Streamfy Home Directory: $STREAMFY_HOME_DIR"

    STREAMFY_BINARIES_DIR="$STREAMFY_HOME_DIR/bin"
    export STREAMFY_BINARIES_DIR
    debug_msg "Streamfy Binaries Directory: $STREAMFY_BINARIES_DIR"

    VERSION_FILE="$(cat ./VERSION)"
    export VERSION_FILE
    debug_msg "Version File Value: $VERSION_FILE"
}

@test "Install svm and setup a settings.toml file" {
    # Ensure the `svm` directory is not present
    run bash -c '! test -d ~/.svm'
    assert_success

    # Installs SVM which introduces the `~/.svm` directory and copies the SVM
    # binary to ~/.svm/bin/svm
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Tests SVM to be in the PATH
    run bash -c 'which svm'
    assert_output --partial ".svm/bin/svm"
    assert_success

    # Retrieves Version from SVM
    run bash -c 'svm --help'
    assert_output --partial "Streamfy Version Manager (SVM)"
    assert_success

    # Ensure the `settings.toml` is available. At this point this is an empty file
    run bash -c 'cat ~/.svm/settings.toml'
    assert_output ""
    assert_success
}

@test "Uninstall svm and removes ~/.svm dir" {
    # Ensure the `svm` directory is present from the previous test
    run bash -c 'test -d ~/.svm'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Test the svm command is present
    run bash -c 'which svm'
    assert_output --partial ".svm/bin/svm"
    assert_success

    # We use `--yes` because prompting is not supported in CI environment,
    # responding with error `Error: IO error: not a terminal`
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Ensure the `~/.svm/` directory is not available anymore
    run bash -c '! test -d ~/.svm'
    assert_success

    # Ensure the svm is not available anymore
    run bash -c '! svm'
    assert_success
}

@test "Creates the `$VERSIONS_DIR` path if not present when attempting to install" {
    # Verify the directory is not present initally
    run bash -c '! test -d $VERSIONS_DIR'
    assert_success

    # Installs SVM as usual
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Renders warn if attempts to use `svm list` and no versions are installed
    run bash -c 'svm list'
    assert_line --index 0 "warn: No installed versions found"
    assert_line --index 1 "help: You can install a Streamfy version using the command svm install"
    assert_success

    # Verify the directory is now present
    run bash -c 'test -d $VERSIONS_DIR'
    assert_success

    # Remove versions directory
    rm -rf $VERSIONS_DIR

    # Installs Stable Streamfy
    run bash -c 'svm install'
    assert_success

    # Checks the presence of the binary in the versions directory
    run bash -c 'test -f $VERSIONS_DIR/stable/streamfy'
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success
}

@test "Install Streamfy Versions" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Expected binaries
    declare -a binaries=(
        streamfy
        streamfy-run
        cdk
        smdk
    )

    # Expected versions
    declare -a versions=(
        $STATIC_VERSION
        stable
        latest
    )

    for version in "${versions[@]}"
    do
        export VERSION="$version"

        run bash -c 'svm install "$VERSION"'
        assert_success

        # Sleeps to avoid hiting rate limits
        sleep 30

        for binary in "${binaries[@]}"
        do
            export BINARY_PATH="$VERSIONS_DIR/$VERSION/$binary"
            echo "Checking binary: $BINARY_PATH"
            run bash -c 'test -f $BINARY_PATH'
            assert_success
        done

        if [ "$VERSION" == "stable" ] || [ "$VERSION" == "latest" ]; then
            run bash -c 'cat "$VERSIONS_DIR/$VERSION/manifest.json" | jq .channel'
            assert_output "\"$version\""
            assert_success
        else
            run bash -c 'cat "$VERSIONS_DIR/$VERSION/manifest.json" | jq .version'
            assert_output "\"$version\""
            assert_success
        fi

        if [ "$VERSION" == "stable" ]; then
            run bash -c '$VERSIONS_DIR/$VERSION/streamfy version > flv_version_$version.out && cat flv_version_$version.out | head -n 1 | grep "$STABLE_VERSION"'
            assert_output --partial "$STABLE_VERSION"
            assert_success
        fi

        if [ "$VERSION" == "$STATIC_VERSION" ]; then
            run bash -c '$VERSIONS_DIR/$VERSION/streamfy version > flv_version_$version.out && cat flv_version_$version.out | head -n 1 | grep "$STATIC_VERSION"'
            assert_output --partial "$STATIC_VERSION"
            assert_success
        fi
    done

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success
}

@test "Copies binaries to Streamfy Binaries Directory when using svm switch" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Removes Streamfy Directory
    rm -rf $STREAMFY_HOME_DIR

    # Ensure `~/.streamfy` is not present
    run bash -c '! test -d $STREAMFY_HOME_DIR'
    assert_success

    declare -a versions=(
        stable
        $STATIC_VERSION
    )

    # Installs Versions
    for version in "${versions[@]}"
    do
        export VERSION="$version"

        # Installs Streamfy Version
        run bash -c 'svm install $VERSION'
        assert_success

        # Sleeps to avoid hiting rate limits
        sleep 30
    done

    for version in "${versions[@]}"
    do
        export VERSION="$version"

        # Switch version to use
        run bash -c 'svm switch $VERSION'
        assert_success

        # Checks version is set
        if [ "$VERSION" == "stable" ]; then
            run bash -c 'streamfy version > flv_version_$version.out && cat flv_version_$version.out | head -n 1 | grep "$STABLE_VERSION"'
            assert_output --partial "$STABLE_VERSION"
            assert_success
        fi

        if [ "$VERSION" == "$STATIC_VERSION" ]; then
            run bash -c 'streamfy version > flv_version_$version.out && cat flv_version_$version.out | head -n 1 | grep "$STATIC_VERSION"'
            assert_output --partial "$STATIC_VERSION"
            assert_success
        fi

        # Checks Settings File's Version is updated
        if [ "$VERSION" == "stable" ]; then
            run bash -c 'yq -oy '.version' "$SETTINGS_TOML_PATH"'
            assert_output --partial "$STABLE_VERSION"
            assert_success
        fi

        if [ "$VERSION" == "$STATIC_VERSION" ]; then
            run bash -c 'yq -oy '.version' "$SETTINGS_TOML_PATH"'
            assert_output --partial "$STATIC_VERSION"
            assert_success
        fi

        # Checks Settings File's Channel is updated
        if [ "$VERSION" == "stable" ]; then
            run bash -c 'yq -oy '.channel' "$SETTINGS_TOML_PATH"'
            assert_output --partial "stable"
            assert_success
        fi

        if [ "$VERSION" == "$STATIC_VERSION" ]; then
            run bash -c 'yq -oy '.channel.tag' "$SETTINGS_TOML_PATH"'
            assert_output --partial "$STATIC_VERSION"
            assert_success
        fi

        # Expected binaries
        declare -a binaries=(
            streamfy
            streamfy-run
            cdk
            smdk
        )

        for binary in "${binaries[@]}"
        do
            export SVM_BIN_PATH="$VERSIONS_DIR/$VERSION/$binary"
            echo "Checking binary: $BINARY_PATH"
            run bash -c 'test -f $SVM_BIN_PATH'
            assert_success

            export FLV_BIN_PATH="$STREAMFY_BINARIES_DIR/$binary"
            echo "Checking binary: $FLV_BIN_PATH"
            run bash -c 'test -f $FLV_BIN_PATH'
            assert_success
        done
    done

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Sets the desired Streamfy Version" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Ensure `~/.streamfy` is not present
    run bash -c '! test -d $STREAMFY_HOME_DIR'
    assert_success

    # Installs Streamfy at 0.10.15
    run bash -c 'svm install 0.10.15'
    assert_success

    # Sleeps to avoid hiting rate limits
    sleep 30

    # Installs Streamfy at 0.10.14
    run bash -c 'svm install 0.10.14'
    assert_success

    # Sleeps to avoid hiting rate limits
    sleep 30

    # Switch version to use Streamfy at 0.10.15
    run bash -c 'svm switch 0.10.15'
    assert_success

    # Checks version is set
    run bash -c 'streamfy version > active_flv_ver.out && cat active_flv_ver.out | head -n 1 | grep "0.10.15"'
    assert_output --partial "0.10.15"
    assert_success

    # Switch version to use Streamfy at 0.10.14
    run bash -c 'svm switch 0.10.14'
    assert_success

    # Checks version is set
    run bash -c 'streamfy version > active_flv_ver.out && cat active_flv_ver.out | head -n 1 | grep "0.10.14"'
    assert_output --partial "0.10.14"
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Keeps track of the active Streamfy Version in settings.toml" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    run bash -c 'rm -rf ~/.streamfy'

    # Ensure `~/.streamfy` is not present
    run bash -c '! test -d $STREAMFY_HOME_DIR'
    assert_success

    # Installs Streamfy Stable
    run bash -c 'svm install stable'
    assert_success

    # Sleeps to avoid hiting rate limits
    sleep 30

    # Installs Streamfy at 0.10.14
    run bash -c 'svm install 0.10.14'
    assert_success

    # Switch version to use Streamfy at Stable
    run bash -c 'svm switch stable'
    assert_success

    # Checks channel is set
    run bash -c 'cat ~/.svm/settings.toml | grep "channel = \"stable\""'
    assert_output --partial "channel = \"stable\""
    assert_success

    # Checks the version is set as active in list list
    run bash -c 'svm list'
    assert_line --index 0 --partial "    CHANNEL  VERSION"
    assert_line --index 1 --partial " ✓  stable   $STABLE_VERSION"
    assert_line --index 2 --partial "    0.10.14  0.10.14"
    assert_success

    # Checks contents for the stable channel
    run bash -c 'svm list stable'
    assert_line --index 0 --partial "Artifacts in channel stable version $STABLE_VERSION"
    assert_output --partial "streamfy@$STABLE_VERSION"
    assert_success

    # Checks contents for the version 0.10.14
    run bash -c 'svm list 0.10.14'
    assert_line --index 0 --partial "Artifacts in version 0.10.14"
    assert_output --partial "streamfy@0.10.14"
    assert_success

    # Checks current command output
    run bash -c 'svm current'
    assert_line --index 0 "$STABLE_VERSION (stable)"
    assert_success

    # Checks version is set
    run bash -c 'cat ~/.svm/settings.toml | grep "version = \"$STABLE_VERSION\""'
    assert_output --partial "version = \"$STABLE_VERSION\""
    assert_success

    # Switch version to use Streamfy at 0.10.14
    run bash -c 'svm switch 0.10.14'
    assert_success

    # Checks version is set
    run bash -c 'cat ~/.svm/settings.toml | grep "version = \"0.10.14\""'
    assert_output --partial "version = \"0.10.14\""
    assert_success

    # Checks channel is tag
    run bash -c 'cat ~/.svm/settings.toml | grep "tag = \"0.10.14\""'
    assert_output --partial "tag = \"0.10.14\""
    assert_success

    # Checks the version is set as active in list
    run bash -c 'svm list'
    assert_line --index 0 --partial "    CHANNEL  VERSION"
    assert_line --index 1 --partial " ✓  0.10.14  0.10.14"
    assert_line --index 2 --partial "    stable   $STABLE_VERSION"
    assert_success

    # Checks current command output
    run bash -c 'svm current'
    assert_line --index 0 "0.10.14"
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Recommends using svm list to list available versions" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    run bash -c 'svm switch'
    assert_line --index 0 "help: You can use svm list to see installed versions"
    assert_line --index 1 "Error: No version provided"
    assert_failure

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success
}

@test "Supress output with '-q' optional argument" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Expect no output if `-q` is passed
    run bash -c 'svm -q install stable'
    assert_output ""
    assert_success

    # Sleeps to avoid hiting rate limits
    sleep 30

    # Expect output if `-q` is not passed
    run bash -c 'svm install 0.10.14'
    assert_line --index 0 --partial "info: Downloading (1/4)"
    assert_output --partial "done: Now using streamfy version 0.10.14"
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Renders help text on current command if none active" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    run bash -c 'svm current'
    assert_line --index 0 "warn: No active version set"
    assert_line --index 1 "help: You can use svm switch to set the active version"
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Sets the desired version after installing it" {
    export VERSION="0.10.14"

    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    run bash -c 'svm current'
    assert_line --index 0 "warn: No active version set"
    assert_line --index 1 "help: You can use svm switch to set the active version"
    assert_success

    run bash -c 'svm install $VERSION'
    assert_success

    run bash -c 'svm current'
    assert_line --index 0 "$VERSION"
    assert_success

    run bash -c 'streamfy version > flv_version_$version.out && cat flv_version_$version.out | head -n 1 | grep "$VERSION"'
    assert_output --partial "$VERSION"
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Replaces binary on installs" {
    # Checks the presence of the binary in the versions directory
    run bash -c '! test -f "$SVM_HOME_DIR/bin/svm"'
    assert_success

    # Create SVM Binaries directory
    mkdir -p "$SVM_HOME_DIR/bin"

    # Create bash file to check if the binary is present
    run bash -c 'echo "echo \"Hello World!\"" > "$SVM_HOME_DIR/bin/svm"'
    assert_success

    # Store checksum of the test binary before install
    export SHASUM_TEST_FILE=$(sha256sum "$SVM_HOME_DIR/bin/svm" | awk '{ print $1 }')

    # Install SVM using `self install`
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Store checksum of the installed SVM binary
    export SHASUM_SVM_BIN=$(sha256sum "$SVM_HOME_DIR/bin/svm" | awk '{ print $1 }')

    # Ensure the checksums are different
    [[ "$SHASUM_TEST_FILE" != "$SHASUM_SVM_BIN" ]]
    assert_success

    # Ensure file is not corrupted on updates
    run bash -c 'svm --help'
    assert_output --partial "Streamfy Version Manager (SVM)"
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success
}

@test "Updating keeps settings files integrity" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    run bash -c 'svm install stable'
    assert_success

    # Store Settings File Checksum
    export SHASUM_SETTINGS_BEFORE=$(sha256sum "$SVM_HOME_DIR/settings.toml" | awk '{ print $1 }')

    # We cannot use `svm self install` so use other copy of SVM to test binary
    # replacement
    run bash -c '$SVM_BIN self install'
    assert_success

    # Store Settings File Checksum
    export SHASUM_SETTINGS_AFTER=$(sha256sum "$SVM_HOME_DIR/settings.toml" | awk '{ print $1 }')

    assert_equal "$SHASUM_SETTINGS_BEFORE" "$SHASUM_SETTINGS_AFTER"

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Fails when using 'svm self install' on itself" {
    skip "Ubuntu CI does not support this test due to mounted virtual volume path mismatch"

    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    run bash -c 'svm install stable'
    assert_success

    # We cannot use `svm self install` so use other copy of SVM to test binary
    # replacement
    run bash -c 'svm self install'
    assert_output --partial "Error: SVM is already installed"
    assert_failure

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Prints version with details on svm version" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    run bash -c 'svm version'
    assert_line --index 0 "svm CLI: $VERSION_FILE"
    assert_line --index 1 --partial "svm CLI Arch: "
    assert_line --index 2 --partial "svm CLI SHA256: "
    assert_line --index 3 --partial "OS Details: "
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Updates version in channel" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Installs the stable version
    run bash -c 'svm install stable'
    assert_success

    # Changes the version set as `stable` channel to $STATIC_VERSION in order to
    # force an update. $STATIC_VERSION just to reuse the variable to improve
    # readability, it could be any version tag (but stable of course!)
    #
    # This wont work in macOS because the sed command is different there
    sed -i "s/$STABLE_VERSION/$STATIC_VERSION/g" $SVM_HOME_DIR/settings.toml

    # Checks active version
    run bash -c 'svm current'
    assert_line --index 0 "$STATIC_VERSION (stable)"
    assert_success

    # Attempts to update Streamfy
    run bash -c 'svm update'
    assert_line --index 0 "info: Updating streamfy stable to version $STABLE_VERSION. Current version is $STATIC_VERSION."
    assert_success

    # Checks active version
    run bash -c 'svm current'
    assert_line --index 0 "$STABLE_VERSION (stable)"
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Do not updates version in static tag" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Installs the stable version
    run bash -c 'svm install $STATIC_VERSION'
    assert_success

    # Attempts to update Streamfy
    run bash -c 'svm update'
    assert_line --index 0 "info: Cannot update a static version tag. You must use a channel."
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Renders message when already up-to-date" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Installs the stable version
    run bash -c 'svm install'
    assert_success

    # Sleeps to avoid hiting rate limits
    sleep 30

    # Attempts to update Streamfy
    run bash -c 'svm update'
    assert_line --index 0 "done: You are already up to date"
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Handles unexistent version error" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Attempts to install unexistent version
    run bash -c 'svm install 0.0.0'
    assert_line --index 0 "Error: Unable to retrieve release for tag v0.0.0: Not Found"
    assert_failure

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Uninstall Versions" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Ensure `~/.svm/versions/stable` is not present
    run bash -c '! test -d $SVM_HOME_DIR/versions/stable'
    assert_success

    # Install stable version
    run bash -c 'svm install stable'
    assert_success

    # Ensure `~/.svm/versions/stable` is present
    run bash -c 'test -d $SVM_HOME_DIR/versions/stable'
    assert_success

    # Uninstall stable version
    run bash -c 'svm uninstall stable'
    assert_success

    # Ensure `~/.svm/versions/stable` is present
    run bash -c '! test -d $SVM_HOME_DIR/versions/stable'
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Updates artifacts in the stable channel" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Install stable version
    run bash -c 'svm install stable'
    assert_success

    # Ensure `~/.svm/versions/stable` is present
    run bash -c 'test -d $SVM_HOME_DIR/versions/stable'
    assert_success

    # Checks for updates
    run bash -c 'svm update'
    assert_line --index 0 "done: You are already up to date"
    assert_success

    # Updates manifest to trigger update
    sed -i -e "s/$STREAMFY_RUN_STABLE_VERSION/0.18.0/g" $SVM_HOME_DIR/versions/stable/manifest.json

    # Removes current `streamfy-run` binary so we check it is re-downloaded
    rm -rf $SVM_HOME_DIR/versions/stable/streamfy-run

    # Ensure `~/.svm/versions/stable/streamfy-run` IS NOT present
    run bash -c '! test -f $SVM_HOME_DIR/versions/stable/streamfy-run'
    assert_success

    # Downloads the update
    run bash -c 'svm update'
    assert_output --partial "info: Updated streamfy-run from 0.18.0 to $STREAMFY_RUN_STABLE_VERSION"
    assert_success

    # Ensure `~/.svm/versions/stable/streamfy-run` IS present
    run bash -c 'test -f $SVM_HOME_DIR/versions/stable/streamfy-run'
    assert_success

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Updates SVM" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Checks installed version
    run bash -c 'svm version'
    assert_line --index 0 "svm CLI: $VERSION_FILE"
    assert_success

    # Store this file Sha256 Checksum
    export CURR_SVM_BIN_CHECKSUM=$(sha256sum "$SVM_HOME_DIR/bin/svm" | awk '{ print $1 }')

    # Store SVM_BIN file Sha256 Checksum
    export SVM_BIN_CHECKSUM=$(sha256sum "$SVM_BIN" | awk '{ print $1 }')

    # Ensure the installed SVM matches test binary SVM
    [[ "$CURR_SVM_BIN_CHECKSUM" == "$SVM_BIN_CHECKSUM" ]]
    assert_success

    if [[ "$SVM_VERSION_STABLE" = "$VERSION_FILE" ]]; then
        # Updates SVM
        run bash -c 'svm self update'
        assert_line --index 0 "info: Already up-to-date"
        assert_success
    else
        # Updates SVM
        run bash -c 'svm self update'
        assert_line --index 0 "info: Updating SVM from $VERSION_FILE to $SVM_VERSION_STABLE"
        assert_line --index 1 "info: Downloading svm@$SVM_VERSION_STABLE"
        assert_line --index 2 "info: Installing svm@$SVM_VERSION_STABLE"
        assert_line --index 3 "done: Installed svm@$SVM_VERSION_STABLE with success"
        assert_success
    fi

    # Removes SVM
    run bash -c 'svm self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Installs Custom SVM Version on SVM Update" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Checks installed version
    run bash -c '$SVM_BIN version'
    assert_line --index 0 "svm CLI: $VERSION_FILE"
    assert_success

    # Updates SVM using SVM as of Streamfy 0.18.0
    run bash -c "SVM_UPDATE_VERSION=$SVM_UPDATE_CUSTOM_VERSION $SVM_BIN self update"
    assert_line --index 0 "info: Updating SVM from $VERSION_FILE to $SVM_UPDATE_CUSTOM_VERSION"
    assert_success

    # Removes SVM
    run bash -c '$SVM_BIN self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}

@test "Supports Binary Target Overriding" {
    run bash -c '$SVM_BIN self install'
    assert_success

    # Sets `svm` in the PATH using the "env" file included in the installation
    source ~/.svm/env

    # Attempts to install unsupported target triple
    run bash -c '$SVM_BIN install 0.11.12 --target aarch64-unknown-linux-gnu'
    assert_line --index 0 "Error: Release \"v0.11.12\" does not have artifacts for architecture: \"aarch64-unknown-linux-gnu\""
    assert_failure

    # Removes SVM
    run bash -c '$SVM_BIN self uninstall --yes'
    assert_success

    # Removes Streamfy
    rm -rf $STREAMFY_HOME_DIR
    assert_success
}
