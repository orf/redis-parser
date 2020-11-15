/// Redis Protocol Parser
///
/// This crate provides a zero-copy parser for the RESP2 and RESP3 protocols.
///
/// # Examples
/// ```
/// use redis_parser::resp2::{parse as parse2, Resp2Type};
/// use redis_parser::resp3::{parse as parse3, Resp3Type};
///
/// assert_eq!(parse2("+test\r\n".as_bytes()), Ok((&b""[..], Resp2Type::String("test"))));
/// assert_eq!(parse3("#f\r\n".as_bytes()), Ok((&b""[..], Resp3Type::Boolean(false))));
/// ```
pub mod resp2;
pub mod resp3;
mod utils;
