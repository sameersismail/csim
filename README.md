# `csim`: Cache Simulator

Simulates a LRU CPU cache.

# Example Usage

```sh
$ head test/trans.trace
 S 00600aa0,1
I  004005b6,5
I  004005bb,5
I  004005c0,5
 S 7ff000398,8
I  0040051e,1
 S 7ff000390,8
I  0040051f,3
I  00400522,4
 S 7ff000378,8

$ ./csim -s 4 -E 1 -b 4 -f ./test/trans.trace
[src/main.rs:38] cache.stats = Statistics {
    hit: 211,
    miss: 27,
    eviction: 18,
}
```

