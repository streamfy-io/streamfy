#!/bin/bash
# This script is ran by the github actions to install streamfy in
# GitHub Action Workflows.

set -eu -o pipefail
echo "Installing Streamfy Local Cluster"

curl -fsS https://raw.githubusercontent.com/streamfy-io/streamfy/main/install.sh | bash
echo 'export PATH="$HOME/.streamfy/bin:$PATH"' >> $HOME/.bash_profile
. $HOME/.bash_profile


LOCAL_FLAG=""
IMAGE=""
#
# Install Streamfy Cluster
#

# Install Local Streamfy Cluster
if [ "$CLUSTER_TYPE" = "local" ]; then
    LOCAL_FLAG="--local"
fi
# Install K8S Streamfy Cluster
if [ "$CLUSTER_TYPE" = "k8" ]; then
    LOCAL_FLAG="--k8"
fi

# For latest, we need to put image tag
if [ "$VERSION" = "latest" ]; then
    IMAGE="--image-version latest"
fi


streamfy cluster start $IMAGE --rust-log $RUST_LOG  $LOCAL_FLAG --spu $SPU_NUMBER
