#!/bin/bash

readonly INTERACTIONS=${INTERACTIONS:-5}

function run_test() {
	${STREAMFY_BIN} cluster delete --force
	${STREAMFY_BIN} cluster start --local

	seq 1 10 | parallel -j 10 ${STREAMFY_BIN} topic create test-topic-{}
	seq 1 10 | parallel -j 10 ${STREAMFY_BIN} remote register test-remote-{}

	${STREAMFY_BIN} cluster shutdown
	${STREAMFY_BIN} cluster resume

	# Create topic
	${STREAMFY_BIN} topic create test-topic-11 # THIS WAS HANGING

	TOPIC_LIST=$(${STREAMFY_BIN} topic list 2>/dev/null)
	PARTITION_LIST=$(${STREAMFY_BIN} partition list 2>/dev/null)
	REMOTE_LIST=$(${STREAMFY_BIN} remote list 2>/dev/null)

	# Check if the topic list has 11+1 lines
	if [ $(echo "$TOPIC_LIST" | wc -l) -eq 12 ]; then
	    echo "PASS"
	else
	    echo "FAIL"
	    exit 1
	fi

	# Check if the partition list has 11+1 lines
	if [ $(echo "$PARTITION_LIST" | wc -l) -eq 12 ]; then
	    echo "PASS"
	else
	    echo "FAIL"
	    exit 1
	fi

	# Check if the remote list has 10+1 lines
	if [ $(echo "$REMOTE_LIST" | wc -l) -eq 11 ]; then
	    echo "PASS"
	else
	    echo "FAIL"
	    exit 1
	fi
}

function main() {
	for i in $(seq 1 $INTERACTIONS);
	do
		echo "INTERATION: $i"
		run_test
	done
}

main;
