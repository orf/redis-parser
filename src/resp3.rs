/// # RESP2
/// This module provides utilities to parse the RESP3 protocol.
use nom::IResult;
use num_bigint::BigInt;

use crate::utils::{
    parse_bytes_with_length, parse_double_with_prefix, parse_integer_with_prefix,
    parse_str_with_length, parse_str_with_prefix,
};
use nom::branch::alt;
use nom::bytes::streaming::{tag, take};
use nom::character::streaming::{char, crlf, digit1};
use nom::combinator::map_res;
use nom::lib::std::str::from_utf8;
use nom::multi::many_m_n;
use nom::sequence::{delimited, tuple};
use std::str::FromStr;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum VerbatimStringFormat {
    Text,
    Markdown,
}

#[derive(Debug, PartialEq)]
pub enum Resp3Type<'a> {
    Blob(&'a [u8]),
    String(&'a str),
    Error(&'a str),
    Number(usize),
    Null,
    Double(f64),
    Boolean(bool),
    BlobError {
        code: &'a str,
        message: &'a str,
    },
    VerbatimString {
        format: VerbatimStringFormat,
        text: &'a str,
    },
    BigNumber(BigInt),

    // Aggregate types
    Array(Vec<Resp3Type<'a>>),

    HashMap(usize),
    Set(usize),
    Attribute(usize),
    Push(Vec<Resp3Type<'a>>),

    StreamArray,
    StreamSet,
    StreamMap,
    StreamEnd,
}

fn parse_blob_string(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, length) = parse_integer_with_prefix(input, '$')?;
    let (input, blob) = parse_bytes_with_length(input, length)?;
    Ok((input, Resp3Type::Blob(blob)))
}

fn parse_simple_string(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, str) = parse_str_with_prefix(input, '+')?;
    Ok((input, Resp3Type::String(str)))
}

fn parse_error(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, str) = parse_str_with_prefix(input, '-')?;
    Ok((input, Resp3Type::Error(str)))
}

fn parse_number(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, num) = parse_integer_with_prefix(input, ':')?;
    return Ok((input, Resp3Type::Number(num)));
}

fn parse_null(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, _) = tag("_\r\n")(input)?;
    Ok((input, Resp3Type::Null))
}

fn parse_double(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, d) = parse_double_with_prefix(input, ',')?;
    Ok((input, Resp3Type::Double(d)))
}

fn parse_boolean(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, b) = delimited(char('#'), alt((char('t'), char('f'))), crlf)(input)?;

    let bool = match b {
        't' => true,
        'f' => false,
        _ => unreachable!(),
    };

    Ok((input, Resp3Type::Boolean(bool)))
}

fn parse_blob_error(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, length) = parse_integer_with_prefix(input, '!')?;
    let (input, blob) = parse_str_with_length(input, length)?;
    let mut splitter = blob.splitn(2, ' ');
    let code = splitter.next().unwrap();
    let message = splitter.next().unwrap_or_else(|| "");
    Ok((input, Resp3Type::BlobError { code, message }))
}

fn parse_verbatim_string(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, length) = parse_integer_with_prefix(input, '=')?;
    let (_, bytes) = parse_bytes_with_length(input, length)?;
    let (input, (format, text)) = map_res(
        tuple((alt((tag("txt"), tag("mkd"))), char(':'), take(length - 4))),
        |(tag, _, contents)| from_utf8(contents).map(|c| (tag, c)),
    )(bytes)?;

    let format = match format {
        b"txt" => VerbatimStringFormat::Text,
        b"mkd" => VerbatimStringFormat::Markdown,
        _ => unreachable!(),
    };

    Ok((input, Resp3Type::VerbatimString { format, text }))
}

fn parse_big_number(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, num) = map_res(delimited(char('('), digit1, crlf), |out| {
        num_bigint::BigInt::from_str(from_utf8(out).unwrap())
    })(input)?;

    Ok((input, Resp3Type::BigNumber(num)))
}

fn parse_array(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, length) = parse_integer_with_prefix(input, '*')?;
    let (input, result) = many_m_n(length, length, parse)(input)?;

    Ok((input, Resp3Type::Array(result)))
}

fn parse_map(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, length) = parse_integer_with_prefix(input, '%')?;
    Ok((input, Resp3Type::HashMap(length)))
}

fn parse_set(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, length) = parse_integer_with_prefix(input, '~')?;
    Ok((input, Resp3Type::Set(length)))
}

fn parse_attribute(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, length) = parse_integer_with_prefix(input, '|')?;
    Ok((input, Resp3Type::Attribute(length)))
}

fn parse_push(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, length) = parse_integer_with_prefix(input, '>')?;
    let (input, result) = many_m_n(length, length, parse)(input)?;

    Ok((input, Resp3Type::Array(result)))
}

fn parse_stream_array(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, _) = tag("*?\r\n")(input)?;
    Ok((input, Resp3Type::StreamArray))
}

fn parse_stream_set(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, _) = tag("~?\r\n")(input)?;
    Ok((input, Resp3Type::StreamSet))
}

fn parse_stream_map(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, _) = tag("%?\r\n")(input)?;
    Ok((input, Resp3Type::StreamMap))
}

fn parse_stream_end(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    let (input, _) = tag(".\r\n")(input)?;
    Ok((input, Resp3Type::StreamEnd))
}

/// Parse bytes into a `Resp3Type` enum
pub fn parse(input: &[u8]) -> IResult<&[u8], Resp3Type> {
    alt((
        parse_blob_string,
        parse_simple_string,
        parse_error,
        parse_number,
        parse_null,
        parse_double,
        parse_boolean,
        parse_blob_error,
        parse_verbatim_string,
        parse_big_number,
        parse_array,
        parse_map,
        parse_set,
        parse_attribute,
        parse_push,
        parse_stream_array,
        parse_stream_set,
        parse_stream_map,
        parse_stream_end,
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::resp3::{parse, Resp3Type, VerbatimStringFormat};
    use num_bigint::BigInt;
    use std::str::FromStr;

    #[test]
    fn test_parse_blob_string() {
        assert_eq!(
            parse(&b"$5\r\n12345\r\n"[..]),
            Ok((&b""[..], Resp3Type::Blob(&b"12345"[..])))
        );
    }

    #[test]
    fn test_parse_simple_string() {
        assert_eq!(
            parse(&b"+12345\r\n"[..]),
            Ok((&b""[..], Resp3Type::String("12345")))
        );
    }

    #[test]
    fn test_parse_error() {
        assert_eq!(
            parse(&b"-ERROR 1234\r\n"[..]),
            Ok((&b""[..], Resp3Type::Error("ERROR 1234")))
        );
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(
            parse(&b":1234\r\n"[..]),
            Ok((&b""[..], Resp3Type::Number(1234)))
        );
    }

    #[test]
    fn test_parse_null() {
        assert_eq!(parse(&b"_\r\n"[..]), Ok((&b""[..], Resp3Type::Null)));
    }

    #[test]
    fn test_parse_double() {
        assert_eq!(
            parse(&b",1.234\r\n"[..]),
            Ok((&b""[..], Resp3Type::Double(1.234f64)))
        );
    }

    #[test]
    fn test_parse_boolean() {
        assert_eq!(
            parse(&b"#t\r\n"[..]),
            Ok((&b""[..], Resp3Type::Boolean(true)))
        );
        assert_eq!(
            parse(&b"#f\r\n"[..]),
            Ok((&b""[..], Resp3Type::Boolean(false)))
        );
    }

    #[test]
    fn test_parse_blob_error() {
        assert_eq!(
            parse(&b"!10\r\nERROR 1234\r\n"[..]),
            Ok((
                &b""[..],
                Resp3Type::BlobError {
                    code: "ERROR",
                    message: "1234"
                }
            ))
        );
    }

    #[test]
    fn test_verbatim_string() {
        assert_eq!(
            parse(&b"=8\r\ntxt:1234\r\n"[..]),
            Ok((
                &b""[..],
                Resp3Type::VerbatimString {
                    format: VerbatimStringFormat::Text,
                    text: "1234"
                }
            ))
        );
        assert_eq!(
            parse(&b"=8\r\nmkd:1234\r\n"[..]),
            Ok((
                &b""[..],
                Resp3Type::VerbatimString {
                    format: VerbatimStringFormat::Markdown,
                    text: "1234"
                }
            ))
        );
    }

    #[test]
    fn test_parse_big_number() {
        assert_eq!(
            parse(&b"(3492890328409238509324850943850943825024385\r\n"[..]),
            Ok((
                &b""[..],
                Resp3Type::BigNumber(
                    BigInt::from_str("3492890328409238509324850943850943825024385").unwrap()
                )
            ))
        );
    }

    #[test]
    fn test_parse_array_empty() {
        assert_eq!(
            parse(&b"*0\r\n"[..]),
            Ok((&b""[..], Resp3Type::Array(vec![])))
        );
    }

    #[test]
    fn test_parse_array_mixed_objecs() {
        assert_eq!(
            parse(&b"*3\r\n:1\r\n:2\r\n$3\r\nfoo\r\n"[..]),
            Ok((
                &b""[..],
                Resp3Type::Array(vec![
                    Resp3Type::Number(1),
                    Resp3Type::Number(2),
                    Resp3Type::Blob(&b"foo"[..])
                ])
            ))
        );
    }

    #[test]
    fn test_parse_map() {
        assert_eq!(parse(&b"%2\r\n"[..]), Ok((&b""[..], Resp3Type::HashMap(2))));
    }

    #[test]
    fn test_parse_set() {
        assert_eq!(parse(&b"~2\r\n"[..]), Ok((&b""[..], Resp3Type::Set(2))));
    }

    #[test]
    fn test_parse_attribute() {
        assert_eq!(
            parse(&b"|2\r\n"[..]),
            Ok((&b""[..], Resp3Type::Attribute(2)))
        );
    }

    #[test]
    fn test_parse_push() {
        assert_eq!(
            parse(&b">2\r\n:1\r\n:2\r\n"[..]),
            Ok((
                &b""[..],
                Resp3Type::Array(vec![Resp3Type::Number(1), Resp3Type::Number(2),])
            ))
        );
    }

    #[test]
    fn test_parse_stream_array() {
        assert_eq!(
            parse(&b"*?\r\n"[..]),
            Ok((&b""[..], Resp3Type::StreamArray))
        );
    }

    #[test]
    fn test_parse_stream_set() {
        assert_eq!(parse(&b"~?\r\n"[..]), Ok((&b""[..], Resp3Type::StreamSet)));
    }

    #[test]
    fn test_parse_stream_map() {
        assert_eq!(parse(&b"%?\r\n"[..]), Ok((&b""[..], Resp3Type::StreamMap)));
    }

    #[test]
    fn test_parse_stream_end() {
        assert_eq!(parse(&b".\r\n"[..]), Ok((&b""[..], Resp3Type::StreamEnd)));
    }
}
