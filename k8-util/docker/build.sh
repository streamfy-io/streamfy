#!/usr/bin/env bash
set -ex

readonly PROGDIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Package streamfy-run into a Docker image
#
# PARAMS:
# $1: The tag to build this Docker image with
#       Ex: 0.7.4-abcdef (where abcdef is a git commit)
# $2: The path to the streamfy-run executable
#       Ex: target/x86_64-unknown-linux-musl/$CARGO_PROFILE/streamfy-run
# $3: Whether to build this Docker image in the Minikube context
#       Ex: true, yes, or anything else that is non-empty
main() {
  local -r target=$1; shift
  local -r commit_hash=$1; shift
  local -r streamfy_run=$1; shift
  local -r K8=$1
  local -r tmp_dir=$(mktemp -d -t streamfy-docker-image-XXXXXX)
  local -r docker_repo="streamfy-io/streamfy"
  local build_args

  if [ "$K8" = "minikube" ]; then
    echo "Setting Minikube build context"
    eval $(minikube -p minikube docker-env --shell=bash)
  fi

  cp "${streamfy_run}" "${tmp_dir}/streamfy-run"
  chmod +x "${tmp_dir}/streamfy-run"
  cp "${PROGDIR}/streamfy.Dockerfile" "${tmp_dir}/Dockerfile"

  if [ "$target" = "aarch64-unknown-linux-musl" ]; then
    local build_args="--build-arg ARCH=arm64v8/"
  fi

  pushd "${tmp_dir}"
  docker build -t "$docker_repo:$commit_hash" -t "$docker_repo:$commit_hash-$target" $build_args .

  if [ "$K8" = "lima" ]; then
    echo "no need to export image for lima"
  fi

  if [ "$K8" = "k3d" ]; then
    echo "export image to k3d cluster"
    docker image save "$docker_repo:$commit_hash" --output /tmp/streamfy.tar
    k3d image import -k /tmp/streamfy.tar -c streamfy
  fi

  if [ "$K8" = "kind" ]; then
    echo "export image to kind cluster"
    docker image save "$docker_repo:$commit_hash" --output /tmp/streamfy.tar
    kind load image-archive /tmp/streamfy.tar
  fi

  if [ "$K8" = "microk8" ]; then
    echo "export image to microk8s cluster"
    # next 2 lines are hack until figure out how to run docker directly on microk8s
    docker image save "$docker_repo:$commit_hash" --output /tmp/streamfy.tar
    multipass transfer /tmp/streamfy.tar microk8s-vm:/tmp/streamfy.tar
    microk8s ctr image import /tmp/streamfy.tar
  fi        

  popd || true
  rm -rf "${tmp_dir}"
}

main "$@"
