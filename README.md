# Redis Protocol Parser
[![Crates.io](https://img.shields.io/crates/v/redis-protocol-parser.svg)](https://crates.io/crates/redis-protocol-parser)
[![Run Tests](https://github.com/orf/redis-protocol-parser/workflows/Run%20Tests/badge.svg)](https://github.com/orf/pinger/redis-protocol-parser)

This library provides a high-performance, zero-copy parser for the RESP2 and RESP3 protocols.

## Usage

There are two simple `parse` functions depending on the protocol you want. This library uses 
the [nom parsing library](https://crates.io/crates/nom) and is built around streaming data into the parser.

```rust
use redis_protocol_parser::resp2::{parse as parse2, Resp2Type};
use redis_protocol_parser::resp3::{parse as parse3, Resp3Type};

assert_eq!(parse2("+test\r\n".as_bytes()), Ok((&b""[..], Resp2Type::String("test"))));
assert_eq!(parse3("#f\r\n".as_bytes()), Ok((&b""[..], Resp3Type::Boolean(false))));
```
