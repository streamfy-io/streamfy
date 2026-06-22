# Smoke Test Scenarios


## 1 SPU

Local Non TLS

```
make RELEASE=true DEFAULT_ITERATION=5000 SERVER_LOG=info smoke-test
```

Local TLS

```
make RELEASE=true DEFAULT_ITERATION=5000 SERVER_LOG=info smoke-test-tls-root
```

K8 Non TLS
```
make RELEASE=true DEFAULT_ITERATION=5000 SERVER_LOG=info smoke-test-k8
```

K8 TLS
```
make RELEASE=true DEFAULT_ITERATION=5000 SERVER_LOG=info smoke-test-k8-tls-root
```

### 2 SPU

Local Non TLS
```
make RELEASE=true DEFAULT_ITERATION=5000 DEFAULT_SPU=2 SERVER_LOG=info smoke-test
```



## With large record size

Iteration: 5000,
Record size: 5k
Log size: 25M

```
flvt --local-driver -p 5000 --record-size 5000 --spu 2 --replication 2
```


## Election Scenario

Create cluster

```
streamfy cluster start --spu 3 --local
``

Create topic with replica 3
```
streamfy topic create -r 3 topic
```

### Produce message

Identity a leader:
```
streamfy partition list
```

Produce a message
```
streamfy produce topic
```

### Read message
```
streamfy consume topic -B -d
```

Kill a leader SPU
```
ps -ef | grep streamfy
```

Verify that SPU is offline
```
streamfy spu list
```

2nd SPU should take over, this should still work:
```
flvd consume topic -B -d
```




