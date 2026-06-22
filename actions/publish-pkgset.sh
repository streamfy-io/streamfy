#!/usr/bin/env bash
set -u

echo Pkgset "$PKGSET_NAME"
echo Streamfy Version "$STREAMFY_VERSION"
echo Streamfy Cloud Version "$STREAMFY_CLOUD_VERSION"

curl -v -X "POST" "https://hub.streamfy.cloud/hub/v1/fvm/pkgset" \
     -H "Authorization: $BPKG_TOKEN" \
     -H 'Content-Type: application/json; charset=utf-8' \
     --data-binary @- << EOF
{
  "artifacts": [
    {
      "name": "streamfy",
      "version": "$STREAMFY_VERSION"
    },
    {
      "name": "streamfy-cloud",
      "version": "$STREAMFY_CLOUD_VERSION"
    },
    {
      "name": "streamfy-run",
      "version": "$STREAMFY_VERSION"
    },
    {
      "name": "cdk",
      "version": "$STREAMFY_VERSION"
    },
    {
      "name": "smdk",
      "version": "$STREAMFY_VERSION"
    }
  ],
  "pkgset": "$PKGSET_NAME"
}
EOF