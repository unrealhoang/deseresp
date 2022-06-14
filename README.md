# Deseresp

[![Build status](https://github.com/unrealhoang/deseresp/actions/workflows/rust.yml/badge.svg)](https://github.com/unrealhoang/deseresp/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/deseresp.svg)](https://crates.io/crates/regex)

Deser-RESP is an implementation of serializer and deserializer for Redis's
[RESP3](https://github.com/redis/redis-specifications/blob/master/protocol/RESP3.md)
format using [serde](https://github.com/serde-rs/serde) framework.

## Example

Simple usage:

```rust
let buf = deseresp::to_vec(&(42, "the Answer")).unwrap();

assert_eq!(&buf, b"*2\r\n:42\r\n+the Answer\r\n");

let source: (usize, &str) = deseresp::from_slice(&buf).unwrap();
assert_eq!(source, (42, "the Answer"));
```

Serialize/Deserialize specific RESP's types with:  
BlobError:
```rust
use deseresp::types::borrowed::BlobError;

let buf = deseresp::to_vec(&(42, BlobError::from("the Answer"))).unwrap();

assert_eq!(&buf, b"*2\r\n:42\r\n!10\r\nthe Answer\r\n");

let source: (usize, BlobError) = deseresp::from_slice(&buf).unwrap();
assert_eq!(source, (42, BlobError::from("the Answer")));
```

Push:
```rust
use deseresp::types::Push;

let buf = deseresp::to_vec(&Push(("message", "channel", "data"))).unwrap();

assert_eq!(&buf, b">3\r\n+message\r\n+channel\r\n+data\r\n");

let source: Push<(&str, &str, &str)> = deseresp::from_slice(&buf).unwrap();
assert_eq!(source.into_inner(), ("message", "channel", "data"));
```

Attribute:
```rust
use deseresp::types::WithAttribute;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TtlMeta {
    ttl: usize
}
let val_with_attr = WithAttribute::new(TtlMeta { ttl: 3600}, 200);
let buf = deseresp::to_vec(&val_with_attr).unwrap();

assert_eq!(&buf, b"|1\r\n+ttl\r\n:3600\r\n:200\r\n");

let source: WithAttribute<TtlMeta, usize> = deseresp::from_slice(&buf).unwrap();
assert_eq!(source.into_inner(), (TtlMeta { ttl: 3600 }, 200));
```

Advance usage, zero-copy network parsing:

```rust
use bytes::{BytesMut, BufMut, Buf};
use serde::Deserialize;

let mut bytes_mut = BytesMut::new();
// loop
{
    // read from network
    bytes_mut.put(&b"*2\r\n:42\r\n+the Answer\r\n"[..]);
    let mut d = deseresp::Deserializer::from_slice(&bytes_mut);
    let r: (usize, &str) = Deserialize::deserialize(&mut d).unwrap();
    // do something with r
    assert_eq!(r, (42, "the Answer"));
    let consumed_bytes = d.get_consumed_bytes();
    // advance bytes_mut
    bytes_mut.advance(consumed_bytes);
}
```
