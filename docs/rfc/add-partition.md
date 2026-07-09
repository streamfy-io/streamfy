# Add partition

### Impact
- https://github.com/streamfy/streamfy/issues/3843

## Overview

Streamfy topics can be set up with multiple partitions in the moment of creation. However, it is a requested feature to add partitions to an existing topic.

## Step-by-step

1. Start a Streamfy cluster with multiple SPUs.

```bash
$ streamfy cluster start --spu 3
```

2. Create a topic with a single partition.

```bash
$ streamfy topic create my-topic
```

3. Verify the initial number of partitions.

```bash
$ streamfy partition list
  NAME      TYPE      PARTITIONS  REPLICAS  RETENTION TIME  COMPRESSION  DEDUPLICATION  STATUS                   REASON
  my-topic  computed  1           1         7days           any          none           resolution::provisioned
```

4. Add a partition to the topic.

```bash
$ streamfy topic add-partition my-topic
added new partitions to topic "my-topic":
PARTITION   SPU
1           5002
```

5. Verify the number of partitions.

```bash
$ streamfy topic list
  NAME      TYPE      PARTITIONS  REPLICAS  RETENTION TIME  COMPRESSION  DEDUPLICATION  STATUS                   REASON
  my-topic  computed  2           1         7days           any          none           resolution::provisioned
```

6. Verify the partition details.

```bash
$ streamfy partition list --topic my-topic
$ streamfy partition list -t my-topic
  TOPIC     PARTITION  LEADER  MIRROR  REPLICAS  RESOLUTION  SIZE  HW  LEO  LRS  FOLLOWER OFFSETS
  my-topic  0          5001            []        Online      0 B   0   0    0    0                 []
  my-topic  1          5002            []        Online      0 B   0   0    0    0                 []
```

## Adding multiples partitions

It's possible specify the number of partitions to add.

```bash
$ streamfy topic add-partition my-topic --count 2
$ streamfy topic add-partition my-topic -c 2
added new partitions to topic "my-topic":
PARTITION   SPU
2           5003
3           5001
```

Check new partitions:

```bash
$ streamfy partition list --topic my-topic
$ streamfy partition list -t my-topic
  TOPIC     PARTITION  LEADER  MIRROR  REPLICAS  RESOLUTION  SIZE  HW  LEO  LRS  FOLLOWER OFFSETS
  my-topic  0          5001            []        Online      0 B   0   0    0    0                 []
  my-topic  1          5002            []        Online      0 B   0   0    0    0                 []
  my-topic  2          5003            []        Online      0 B   0   0    0    0                 []
  my-topic  3          5001            []        Online      0 B   0   0    0    0                 []
``

## Produce messages

Streamfy producers must produce messages to a new partition, even if this partition were created after the produce connection.

1. Produce messages to the topic.

```bash
$ streamfy create my-topic
$ streamfy produce my-topic
> A
Ok!
> B
Ok!
$ streamfy partition list
TOPIC     PARTITION  LEADER  MIRROR  REPLICAS  RESOLUTION  SIZE   HW  LEO  LRS  FOLLOWER OFFSETS
my-topic  0          5001            []        Online      138 B  2   2    2    0                 []
```

2. Add a new partition to the topic.

```bash
$ streamfy topic add-partition my-topic
```

Back to the producer, it must produce messages to the new partition.

```bash
> C
Ok!
> D
Ok!

$ streamfy partition list
TOPIC     PARTITION  LEADER  MIRROR  REPLICAS  RESOLUTION  SIZE   HW  LEO  LRS  FOLLOWER OFFSETS
my-topic  0          5001            []        Online      207 B  3   3    3    0                 []
my-topic  1          5001            []        Online      69 B   1   1    1    0                 []
```

## Consume messages

Streamfy consumers must to consume messages from a new partition when consuming a topic reading from all partition.
Based on the previous example, if we start to the consumer command before the new partition is added, it must read all messages, even from the new partition.

```bash
$ streamfy consume my-topic
A
C
B
D
```

## Partition rebalancing

Adding a partition will not rebalance the data from the existing partitions. The rebalancing will happen naturally after the records age.

The new data will be distributed across all partitions, including the newly added partitions.
