STREAMFY_BIN=target/debug/streamfy
SPU=2
REPLICATION=2

streamfy:
	cargo build --bin streamfy

# make -f src/spu/Makefile test_replica
test_replica:	streamfy
	$(STREAMFY_BIN) cluster delete --force
	$(STREAMFY_BIN) cluster start --spu $(SPU) --local --develop --rust-log=streamfy_spu=debug
	$(STREAMFY_BIN) topic create test --replication $(REPLICATION)
	echo "hello world" | $(STREAMFY_BIN) produce test
	sleep 2
	$(STREAMFY_BIN) partition list




