#!/bin/bash
# delete and re-install k3d cluster ready for streamfy
# this defaults to docker and assume you have have sudo access
set -e

K8_VERSION=${K8_VERSION:-v1.26.3}

k3d cluster delete streamfy
k3d cluster create streamfy --image rancher/k3s:${K8_VERSION}-k3s1