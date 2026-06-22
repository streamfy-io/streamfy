#/bin/bash
# generate large data size
set -e
streamfy topic create t1
streamfy produce t1 -r $(ls /var/log/journal/**/*.journal)