use nom::branch::alt;
use nom::multi::many_m_n;
use nom::{bytes::streaming::tag, IResult};

use crate::utils::{parse_bytes_with_length, parse_integer_with_prefix, parse_str_with_prefix};

#[derive(Debug, PartialEq)]
pub enum Resp2Type<'a> {
    String(&'a str),
    Error(&'a str),
    Integer(usize),
    BulkString(&'a [u8]),
    Null,
    Array(Vec<Resp2Type<'a>>),
}

fn parse_simple_string(input: &[u8]) -> IResult<&[u8], Resp2Type> {
    let (input, string) = parse_str_with_prefix(input, '+')?;

    Ok((input, Resp2Type::String(string)))
}

fn parse_error(input: &[u8]) -> IResult<&[u8], Resp2Type> {
    let (input, string) = parse_str_with_prefix(input, '-')?;

    Ok((input, Resp2Type::Error(string)))
}

fn parse_integer(input: &[u8]) -> IResult<&[u8], Resp2Type> {
    let (input, int) = parse_integer_with_prefix(input, ':')?;

    Ok((input, Resp2Type::Integer(int)))
}

fn parse_bulk_string(input: &[u8]) -> IResult<&[u8], Resp2Type> {
    let (input, length) = parse_integer_with_prefix(input, '$')?;
    let (input, bytes) = parse_bytes_with_length(input, length)?;

    Ok((input, Resp2Type::BulkString(bytes)))
}

fn parse_null_string(input: &[u8]) -> IResult<&[u8], Resp2Type> {
    let (input, _) = tag("$-1\r\n")(input)?;

    Ok((input, Resp2Type::Null))
}

fn parse_array(input: &[u8]) -> IResult<&[u8], Resp2Type> {
    let (input, length) = parse_integer_with_prefix(input, '*')?;
    let (input, result) = many_m_n(length, length, parse)(input)?;

    Ok((input, Resp2Type::Array(result)))
}

pub fn parse(input: &[u8]) -> IResult<&[u8], Resp2Type> {
    alt((
        parse_simple_string,
        parse_error,
        parse_integer,
        parse_bulk_string,
        parse_null_string,
        parse_array,
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::resp2::{parse, Resp2Type};

    #[test]
    fn test_parse_simple_string() {
        assert_eq!(
            parse(&b"+OK\r\n"[..]),
            Ok((&b""[..], Resp2Type::String("OK")))
        );
    }

    #[test]
    fn test_parse_error() {
        assert_eq!(
            parse(&b"-Error message\r\n"[..]),
            Ok((&b""[..], Resp2Type::Error("Error message")))
        );
    }

    #[test]
    fn test_parse_integer() {
        assert_eq!(
            parse(&b":100\r\n"[..]),
            Ok((&b""[..], Resp2Type::Integer(100)))
        );
    }

    #[test]
    fn test_parse_bulk_string() {
        assert_eq!(
            parse(&b"$10\r\n1234567890\r\n"[..]),
            Ok((&b""[..], Resp2Type::BulkString(&b"1234567890"[..])))
        );
    }

    #[test]
    fn test_parse_null_string() {
        assert_eq!(parse(&b"$-1\r\n"[..]), Ok((&b""[..], Resp2Type::Null)));
    }

    #[test]
    fn test_parse_array_empty() {
        assert_eq!(
            parse(&b"*0\r\n"[..]),
            Ok((&b""[..], Resp2Type::Array(vec![])))
        );
    }

    #[test]
    fn test_parse_array_mixed_objecs() {
        assert_eq!(
            parse(&b"*3\r\n:1\r\n:2\r\n$3\r\nfoo\r\n"[..]),
            Ok((
                &b""[..],
                Resp2Type::Array(vec![
                    Resp2Type::Integer(1),
                    Resp2Type::Integer(2),
                    Resp2Type::BulkString(&b"foo"[..])
                ])
            ))
        );
    }
}
