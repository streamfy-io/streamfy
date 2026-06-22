#!/bin/bash

kubectl logs  "$@" `(kubectl get pod -l app=streamfy-sc  -o jsonpath="{.items[0].metadata.name}")`