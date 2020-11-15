/// Redis Protocol Parser
///
/// This crate provides a zero-copy parser for the RESP2 and RESP3 protocols.
///
/// # Examples
/// ```
/// use redis_protocol_parser::resp2::{parse, Resp2Type};
///
/// assert_eq!(parse("+test\r\n".as_bytes()), Ok((&b""[..], Resp2Type::String("test"))));
/// ```
pub mod resp2;
pub mod resp3;
mod utils;
